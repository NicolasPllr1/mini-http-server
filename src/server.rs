use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::thread_pool::ThreadPool;
use std::time::Duration;

use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub struct Server {
    pub address: String,
    pub thread_pool: ThreadPool,
    pub data_dir: Arc<String>, // NOTE: Arc vs plain String: pblm with Arc::Clone in run()
}

impl Server {
    #[must_use]
    pub fn new(address: &str, pool_size: usize, data_dir: &str) -> Self {
        Server {
            address: address.to_string(),
            thread_pool: ThreadPool::new(pool_size),
            data_dir: data_dir.to_string().into(),
        }
    }

    /// Start the server running.
    ///
    /// # Errors
    ///
    /// Returns an error when an incoming TCP connection can't be accepted
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.address)?;
        let pool = &self.thread_pool;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let data_dir = Arc::clone(&self.data_dir); // NOTE: self vs Self vs Server
                    pool.execute(move || {
                        match Self::handle_stream(stream, &data_dir) {
                            Ok(()) => println!("Successfully handled stream"),
                            Err(e) => eprintln!("Error handling the stream: {e}"), // TODO: propagate
                                                                                   // the error to the main thread ?
                        };
                    });
                }
                Err(e) => {
                    return Err(format!("Error accepting the connection: {e}").into());
                }
            }
        }
        Ok(())
    }

    fn handle_stream(mut stream: TcpStream, data_dir: &str) -> Result<(), Box<dyn Error>> {
        println!("accepted new connection");
        stream.set_read_timeout(Some(Duration::new(30, 0)))?; // 30s

        // TODO: if build_from_stream err, then we build error-404 reponse ? always want to answer
        // I guess

        let mut keep_alive = true;

        while keep_alive {
            let http_request = HttpRequest::build_from_stream(&mut stream)?;

            keep_alive = http_request.keep_alive();
            println!("keep-alive: {keep_alive}");

            println!("parsed http-request: {http_request:?}");

            let http_response = HttpResponse::build_from_request(&http_request, data_dir)?;
            println!("built http-response: {http_response:?}");

            http_response.write_to(&mut stream)?;
        }

        Ok(())
    }
}
