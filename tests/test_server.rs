use flyweight_http_server;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[cfg(test)]
struct TestServer {
    port: u16,
    _server_thread: thread::JoinHandle<()>, // We prefix with _ to indicate we won't join it
}

#[cfg(test)]
impl TestServer {
    fn start() -> Self {
        let port = 4221;
        let server_thread = thread::spawn(move || {
            // let args = vec![String::from("program"), String::from("")];
            std::env::set_var("CARGO_MANIFEST_DIR", ".");
            flyweight_http_server::main();
        });

        // Wait for server to start
        thread::sleep(Duration::from_millis(100));

        TestServer {
            port,
            _server_thread: server_thread,
        }
    }

    fn send_request(&self, path: &str) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost:{}\r\n\r\n",
            path, self.port
        );
        stream.write_all(request.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn send_request_with_headers(&self, path: &str, headers: &[(&str, &str)]) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();

        let mut request = format!("GET {} HTTP/1.1\r\nHost: localhost:{}\r\n", path, self.port);

        // Add custom headers
        for (name, value) in headers {
            request.push_str(&format!("{}: {}\r\n", name, value));
        }
        request.push_str("\r\n");

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}

#[test]
fn test_echo_endpoint() {
    let server = TestServer::start();
    let response = server.send_request("/echo/hello");

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("content-type: text/plain"));
    assert!(response.contains("hello"));
}

#[test]
fn test_user_agent() {
    let server = TestServer::start();
    let headers = &[("User-Agent", "test-agent")];
    let response = server.send_request_with_headers("/user-agent", headers);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("test-agent"));
}

#[test]
fn test_not_found() {
    let server = TestServer::start();
    let response = server.send_request("/nonexistent");

    assert!(response.contains("HTTP/1.1 404 Not Found"));
}
