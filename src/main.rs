use std::env;
use std::error::Error;

use flyweight_http_server::Builder;
use flyweight_http_server::Server;

//TODO:
// 1. args config, with proper parsing like in the Book
// 2. handle error here aswell ? like in the Book

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let cli_cfg = Builder::from_cli_args(&args)?;
    let file_cfg = Builder::from_config_file("server_config.toml")?;
    let env_cfg = Builder::from_env()?;

    let cfg = cli_cfg.merge(&file_cfg).merge(&env_cfg).build();

    println!("Config: {cfg:?}");

    let server = Server::new(&cfg.server_addr, cfg.pool_size, &cfg.data_dir);

    server.run()?;

    println!("Shutting down.");

    Ok(())
}
