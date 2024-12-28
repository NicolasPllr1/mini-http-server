mod thread_pool;
use thread_pool::ThreadPool;
mod html_request;
use html_request::HttpRequest;

mod html_response;
use html_response::HttpResponse;

mod html_commons;

use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn handle_stream(mut stream: TcpStream) {
    println!("accepted new connection");

    let http_request = HttpRequest::new_from_stream(&mut stream);
    // dbg!(&http_request);

    let http_response = HttpResponse::build_response(&http_request);
    // dbg!(&http_response);
    let final_response = http_response.to_string();
    dbg!(&final_response);
    let _res = stream.write(&final_response.into_bytes());
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_stream(stream);
                });
            }
            Err(e) => {
                println!("Error accepting the connection: {}", e);
            }
        }
    }
    println!("Shutting down.");
}
