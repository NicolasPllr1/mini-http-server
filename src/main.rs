#[allow(unused_imports)]
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn is_html_request_complete(buf: &Vec<u8>) -> bool {
    let end_pattern = "\r\n\r\n"; // double CRFL
    buf.windows(end_pattern.len()).any(|w| w == b"\r\n\r\n")
}

fn read_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let mut tmp_buf = [0; 1024];
    loop {
        match stream.read(&mut tmp_buf) {
            Ok(n) => {
                buf.extend_from_slice(&tmp_buf[..n]);
                if is_html_request_complete(&buf) {
                    return Some(buf);
                }
            }
            Err(e) => {
                println!("Problem reading the request: {}", e);
                return None;
            }
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let ok_response = b"HTTP/1.1 200 OK\r\n\r\n";

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let buf: Vec<u8> = read_request(&mut stream).unwrap();
                let full_request = String::from_utf8(buf).expect("Our bytes should be valid utf8");
                println!("Request: {}", full_request);
                let request_line = full_request.split("\n").collect::<Vec<&str>>()[0];
                dbg!(request_line);
                if *"GET /" == request_line[..5] {
                    let res = stream.write(ok_response);
                    match res {
                        Ok(_) => println!("Successfully sent 200 OK response"),
                        Err(e) => println!("Error sending 200 OK response: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("Error accepting the connection: {}", e);
            }
        }
    }
}
