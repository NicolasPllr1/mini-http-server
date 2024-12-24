mod html_request;
use html_request::HttpRequest;

mod html_response;
use html_response::HttpResponse;

mod html_commons;

use std::io::Write;
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let ok_response = b"HTTP/1.1 200 OK\r\n\r\n";
    // let not_ok_response = b"HTTP/1.1 404 Not Found\r\n\r\n";

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let http_request = HttpRequest::new_from_stream(&mut stream);
                dbg!(&http_request);

                if *"/" == http_request.ressource_path {
                    let res = stream.write(ok_response);
                    match res {
                        Ok(_) => println!("Successfully sent 200 OK response"),
                        Err(e) => println!("Error sending 200 OK response: {}", e),
                    }
                } else {
                    let http_response = HttpResponse::build_response(&http_request).unwrap();
                    let final_response_to_send = HttpResponse::craft_response(&http_response);
                    let res = stream.write(final_response_to_send.as_bytes());
                    match res {
                        Ok(_) => println!("Successfully sent response: {}", final_response_to_send),
                        Err(e) => println!("Error sending  response: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("Error accepting the connection: {}", e);
            }
        }
    }
}
