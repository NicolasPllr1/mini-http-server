use std::env;

use flyweight_http_server::server::Server;

//TODO:
// 1. args config, with proper parsing like in the Book
// 2. handle error here aswell ? like in the Book

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let data_dir = match args.len() {
        2 => args[1].clone(),
        3 => args[2].clone(),
        _ => String::new(),
    };
    let address = "127.0.0.1:4221";
    let pool_size = 10;

    let server = Server::new(address, pool_size, &data_dir);

    let _ = server.run();

    println!("Shutting down.");
}
