use flyweight_http_server::Server;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub struct TestServer {
    address: SocketAddr,
}

impl TestServer {
    pub fn new() -> Self {
        // Use a fixed port for simplicity, or choose a random available port
        let port = 4221;
        let address = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap();

        TestServer { address }
    }

    pub fn run(&self) {
        // Clone the address for the thread
        let address = self.address;
        let data_dir = PathBuf::from_str("test_data").unwrap();

        // Start server in background thread
        thread::spawn(move || {
            let server = Server::new(&address, 4, &data_dir);
            // This runs in an infinite loop
            let _ = server.run();
        });

        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));
    }

    pub fn send_request(&self, path: &str) -> String {
        // Create the HTTP request
        let request = format!("GET {} HTTP/1.1\r\nHost: localhost\r\n\r\n", path);

        // Connect to the server and send the request
        let mut stream = TcpStream::connect(&self.address).unwrap();
        stream.write_all(request.as_bytes()).unwrap();

        // Wait 1sec before reading the response
        std::thread::sleep(std::time::Duration::from_secs(1));
        // Read the response
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer[0..n]).to_string()
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
        println!("Response:\n{}", response);

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: text/plain"));
        assert!(response.contains("hello"));
    }
}
