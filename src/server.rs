use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::thread_pool::ThreadPool;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub struct Server {
    pub address: SocketAddr,
    pub thread_pool: ThreadPool,
    pub data_dir: Arc<Path>, // NOTE: Arc vs plain String: pblm with Arc::Clone in run()
}

impl Server {
    #[must_use]
    pub fn new(address: &SocketAddr, pool_size: usize, data_dir: &Path) -> Self {
        Server {
            address: *address,
            thread_pool: ThreadPool::new(pool_size),
            data_dir: Arc::from(data_dir),
        }
    }

    /// Start the server running.
    ///
    /// # Errors
    ///
    /// Returns an error when an incoming TCP connection can't be accepted
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(self.address)?;
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

    fn handle_stream(mut stream: TcpStream, data_dir: &Path) -> Result<(), Box<dyn Error>> {
        println!("accepted new connection");
        stream.set_read_timeout(Some(Duration::new(30, 0)))?; // 30s

        // TODO: if build_from_stream err, then we build error-404 reponse ? always want to answer
        // I guess

        let mut keep_alive = true;

        while keep_alive {
            match HttpRequest::build_from_stream(&mut stream) {
                Ok(http_request) => {
                    println!("parsed http-request: {http_request:?}");

                    keep_alive = http_request.keep_alive();
                    println!("keep-alive: {keep_alive}");

                    let http_response = HttpResponse::new_from_request(&http_request, data_dir);
                    println!("built http-response: {http_response:?}");
                    http_response.write_to(&mut stream)?;
                }
                Err(e) => {
                    eprintln!("error parsing the http-request: {e}");
                    keep_alive = false; // terminate connection
                    let http_response = HttpResponse::new_from_bad_request(&e);
                    println!("built http-response: {http_response:?}");
                    http_response.write_to(&mut stream)?;
                }
            }
        }

        Ok(())
    }
}
