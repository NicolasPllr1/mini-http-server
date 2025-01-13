use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::thread_pool::ThreadPool;

use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::usize;

pub struct Server {
    address: String,
    thread_pool: ThreadPool,
    data_dir: Arc<String>, //NOTE: Arc vs plain String, pblm in Arc::Clone in run()
}

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
                        match Self::handle_stream(stream, Arc::clone(&data_dir)) {
                            Ok(()) => println!("Successfully handled stream"),
                            Err(e) => println!("Error handling the stream: {}", e),
                        }; //NOTE: self vs Self vs
                           //Server
                    });
                }
                Err(e) => {
                    println!("Error accepting the connection: {}", e);
                }
            }
        }
    }

    fn handle_stream(mut stream: TcpStream, data_dir: Arc<String>) -> Result<(), Box<dyn Error>> {
        println!("accepted new connection");

        let http_request = HttpRequest::build_from_stream(&mut stream)?;

        let http_response = HttpResponse::new_response(&http_request, &data_dir);

        let _ = http_response.write_to(&mut stream)?;
        Ok(())
    }
}
