use std::env;
use std::error::Error;

use flyweight_http_server::Server;

//TODO:
// 1. args config, with proper parsing like in the Book
// 2. handle error here aswell ? like in the Book

#[derive(Debug, Clone)]
struct Config {
    server_addr: String,
    pool_size: usize,
    data_dir: String, // TODO: change to Option<String> to signal abscence of data_dir set ?
}

struct ConfigBuilder {
    server_addr: Option<String>,
    pool_size: Option<usize>,
    data_dir: Option<String>,
}

impl ConfigBuilder {
    fn new() -> ConfigBuilder {
        ConfigBuilder {
            server_addr: None,
            pool_size: None,
            data_dir: None,
        }
    }

    fn build(self) -> Config {
        Config {
            server_addr: self
                .server_addr
                .unwrap_or_else(|| "127.0.0.1:4221".to_string()),
            pool_size: self.pool_size.unwrap_or(10),
            data_dir: self.data_dir.unwrap_or_else(|| "./data".to_string()),
        }
    }

    fn from_cli_args(args: &[String]) -> Self {
        let mut builder = Self::new();
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--address" | "-a" => {
                    if let Some(addr) = iter.next() {
                        builder.server_addr = Some(addr.to_string());
                    }
                }
                "--pool-size" | "-s" => {
                    if let Some(size) = iter.next() {
                        builder.pool_size = Some(size.parse().unwrap());
                    }
                }
                "--data-dir" | "-d" => {
                    if let Some(dir) = iter.next() {
                        builder.data_dir = Some(dir.to_string());
                    }
                }
                _ => {
                    eprintln!("CLI argument not recognize: {arg}");
                }
            }
        }

        builder
    }

    fn from_env() -> Self {
        let mut builder = Self::new();

        if let Ok(val) = std::env::var("ADDRESS") {
            builder.server_addr = Some(val);
        }
        if let Ok(val) = std::env::var("POOL_SIZE") {
            builder.pool_size = Some(val.parse().unwrap());
        }
        if let Ok(val) = std::env::var("DATA_DIR") {
            builder.data_dir = Some(val);
        }

        builder
    }
    fn from_config_file(cfg_path: &str) -> Self {
        use std::fs;
        let content = fs::read_to_string(cfg_path).unwrap(); // TODO: if config file does not exist

        let mut builder = Self::new();

        let mut in_server_section: bool = false;

        for line in content.lines() {
            if line.starts_with('[') && line.ends_with(']') & line.contains("server") {
                in_server_section = true;
            }

            if in_server_section {
                if let Some((cfg_key, cfg_value)) = line.split_once('=') {
                    let cfg_key = cfg_key.trim();
                    let cfg_value = cfg_value.trim();
                    match cfg_key {
                        "address" => builder.server_addr = Some(cfg_value.to_string()),
                        "pool_size" => builder.pool_size = Some(cfg_value.parse().unwrap()),
                        "data_dir" => builder.data_dir = Some(cfg_value.to_string()),
                        _ => eprintln!("Warning: unknown key-value pair found in con)fig file [server] section: {cfg_key} = {cfg_value}"),
                    }
                }
            }
        }

        builder
    }

    fn merge(&self, other: &ConfigBuilder) -> ConfigBuilder {
        ConfigBuilder {
            server_addr: self.server_addr.clone().or(other.server_addr.clone()),
            pool_size: self.pool_size.or(other.pool_size), // NOTE: usize is Copy, no clone needed
            data_dir: self.data_dir.clone().or(other.data_dir.clone()),
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let cli_cfg = ConfigBuilder::from_cli_args(&args);
    let file_cfg = ConfigBuilder::from_config_file("server_config.toml");
    let env_cfg = ConfigBuilder::from_env();

    let cfg = cli_cfg.merge(&file_cfg).merge(&env_cfg).build();

    println!("Config: {cfg:?}");

    let server = Server::new(&cfg.server_addr, cfg.pool_size, &cfg.data_dir);

    server.run()?;

    println!("Shutting down.");

    Ok(())
}
