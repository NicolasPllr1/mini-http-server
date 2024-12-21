#[allow(unused_imports)]
use std::io::prelude::*;
use std::net::TcpListener;

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
                let res = stream.write(ok_response);
                match res {
                    Ok(_) => println!("Successfully sent 200 OK response"),
                    Err(e) => println!("Error sending 200 OK response: {}", e),
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
