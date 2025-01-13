use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::thread_pool::ThreadPool;

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::usize;

pub struct Server {
    address: String,
    thread_pool: ThreadPool,
    data_dir: Arc<String>, //NOTE: Arc vs plain String, pblm in Arc::Clone in run()
}

// let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

impl Server {
    pub fn new(address: String, pool_size: usize, data_dir: String) -> Self {
        Server {
            address,
            thread_pool: ThreadPool::new(pool_size),
            data_dir: data_dir.into(),
        }
    }
    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        let pool = &self.thread_pool;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let data_dir = Arc::clone(&self.data_dir);
                    pool.execute(move || {
                        Self::handle_stream(stream, Arc::clone(&data_dir)); //NOTE: self vs Self vs
                                                                            //Server
                    });
                }
                Err(e) => {
                    println!("Error accepting the connection: {}", e);
                }
            }
        }
    }

    fn handle_stream(mut stream: TcpStream, data_dir: Arc<String>) {
        println!("accepted new connection");

        let http_request = HttpRequest::new_from_stream(&mut stream);
        dbg!(&http_request);

        let http_response = HttpResponse::build_response(&http_request, &data_dir);
        dbg!(&http_response);
        dbg!(http_response.to_string());

        let _ = http_response.write_to(&mut stream);
    }
}
