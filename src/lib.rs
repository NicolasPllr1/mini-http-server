use std::env;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub mod encoding;
pub mod http_commons;
pub mod http_request;
pub mod http_response;
pub mod thread_pool;

use http_request::HttpRequest;
use http_response::HttpResponse;
use thread_pool::ThreadPool;

fn handle_stream(mut stream: TcpStream, data_dir: Arc<String>) {
    println!("accepted new connection");

    let http_request = HttpRequest::new_from_stream(&mut stream);
    dbg!(&http_request);

    let http_response = HttpResponse::build_response(&http_request, &data_dir);
    dbg!(&http_response);
    dbg!(http_response.to_string());

    let _ = http_response.write_to(&mut stream);
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let data_dir = Arc::new(match args.len() {
        2 => args[1].clone(),
        3 => args[2].clone(),
        _ => String::new(),
    });

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let data_dir = Arc::clone(&data_dir);
                pool.execute(move || {
                    handle_stream(stream, Arc::clone(&data_dir));
                });
            }
            Err(e) => {
                println!("Error accepting the connection: {}", e);
            }
        }
    }
    println!("Shutting down.");
}
