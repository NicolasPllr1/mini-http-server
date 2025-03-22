use flyweight_http_server::Server;

use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

struct TestServer {
    server: Server,
}

impl TestServer {
    fn new() -> Self {
        let address = "127.0.0.1:4221";
        let pool_size = 10; // thread pool size
        let data_dir = "data/";
        TestServer {
            server: Server::new(address, pool_size, data_dir),
        }
    }
    fn run(&self) -> Result<(), Box<dyn Error>> {
        self.server.run()
    }

    fn send_request(self, path: &str) -> String {
        let mut stream = TcpStream::connect(self.server.address).unwrap();

        let request = format!("GET {} HTTP/1.1\r\nHost: localhost:4221\r\n\r\n", path);
        stream.write_all(request.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn send_request_with_headers(self, path: &str, headers: &[(&str, &str)]) -> String {
        let mut stream = TcpStream::connect(self.server.address).unwrap();

        let mut request = format!("GET {} HTTP/1.1\r\nHost: localhost:4221\r\n", path);

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

mod test {
    use crate::TestServer;

    #[test]
    fn test_echo_endpoint() {
        let server = TestServer::new();
        let _ = server.run();

        let path = "/echo/hello";
        let response = server.send_request(path);

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("content-type: text/plain"));
        assert!(response.contains("hello"));
    }

    #[test]
    fn test_user_agent() {
        let server = TestServer::new();
        let _ = server.run();

        let headers = &[("User-Agent", "test-agent")];
        let path = "/user-agent";
        let response = server.send_request_with_headers(path, headers);

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("test-agent"));
    }

    #[test]
    fn test_not_found() {
        let server = TestServer::new();
        let _ = server.run();

        let response = server.send_request("/nonexistent");

        assert!(response.contains("HTTP/1.1 404 Not Found"));
    }
}
