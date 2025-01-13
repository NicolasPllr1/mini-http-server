use std::env;

use flyweight_http_server::server::Server;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let data_dir = match args.len() {
        2 => args[1].clone(),
        3 => args[2].clone(),
        _ => String::new(),
    };
    let address = "127.0.0.1:4221";
    let pool_size = 10;

    let server = Server::new(address.to_string(), pool_size, data_dir);

    server.run();

    println!("Shutting down.");
}
