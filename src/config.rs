use std::{
    fmt, fs,
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
    num::ParseIntError,
    path::PathBuf,
};

#[derive(Debug)]
pub struct Config {
    pub server_addr: SocketAddr,
    pub pool_size: usize,
    pub data_dir: PathBuf, // PathBuf vs Path
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ConfigError {
    PoolSizeZero,
    PoolSizeParseError(ParseIntError),
    BadServerAddr(AddrParseError),
    DataDirDoesNotExists,
    DataDirIoError(std::io::Error),
    UnknownFlag(String),
    MissingValue(&'static str),
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> ConfigError {
        ConfigError::PoolSizeParseError(e)
    }
}
impl From<AddrParseError> for ConfigError {
    fn from(e: AddrParseError) -> ConfigError {
        ConfigError::BadServerAddr(e)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> ConfigError {
        ConfigError::DataDirIoError(e)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config error: {self:?}")
    }
}

impl std::error::Error for ConfigError {} // empty impl ?!

pub struct Builder {
    server_addr: Option<SocketAddr>,
    pool_size: Option<usize>,
    data_dir: Option<PathBuf>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            server_addr: None,
            pool_size: None,
            data_dir: None,
        }
    }

    #[must_use]
    pub fn build(self) -> Config {
        let default_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4221);
        let default_data_dir = PathBuf::from("."); // PathBuf::from("data")
        Config {
            server_addr: self.server_addr.unwrap_or(default_socket),
            pool_size: self.pool_size.unwrap_or(10),
            data_dir: self.data_dir.unwrap_or(default_data_dir),
        }
    }

    /// Config builder from CLI args
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` variant
    pub fn from_cli_args(args: &[String]) -> Result<Self, ConfigError> {
        let mut builder = Self::new();
        let mut iter = args.iter().peekable();
        iter.next(); // executable path
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--address" | "-a" => {
                    let addr = iter
                        .next()
                        .ok_or(ConfigError::MissingValue("--address"))?
                        .parse::<SocketAddr>()?;
                    builder.server_addr = Some(addr);
                }
                "--pool-size" | "-s" => {
                    let size = iter
                        .next()
                        .ok_or(ConfigError::MissingValue("--pool-size"))?
                        .parse()?;

                    if size == 0 {
                        return Err(ConfigError::PoolSizeZero);
                    }

                    builder.pool_size = Some(size);
                }
                "--data-dir" | "--directory" | "-d" => {
                    let dir_path = iter.next().ok_or(ConfigError::MissingValue("--data-dir"))?;
                    let dir_path = fs::canonicalize(dir_path)?; // no need for mut ?! for
                                                                // shadowing here ?
                    match fs::exists(&dir_path) {
                        Ok(true) => builder.data_dir = Some(dir_path),
                        Ok(false) => return Err(ConfigError::DataDirDoesNotExists),
                        Err(e) => return Err(ConfigError::DataDirIoError(e)),
                    }
                }
                _ => {
                    return Err(ConfigError::UnknownFlag(format!(
                        "Unknown CLI argument flag: {arg}"
                    )));

                    // TODO: detect duplicate args
                }
            }
        }

        Ok(builder)
    }

    /// Config builder from env. variables
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` variant
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Ok(val) = std::env::var("ADDRESS") {
            let addr = val.parse::<SocketAddr>()?;
            builder.server_addr = Some(addr);
        }
        if let Ok(val) = std::env::var("POOL_SIZE") {
            let size = val.parse::<usize>()?;
            if size == 0 {
                return Err(ConfigError::PoolSizeZero);
            }
            builder.pool_size = Some(size);
        }
        if let Ok(val) = std::env::var("DATA_DIR") {
            let dir_path = fs::canonicalize(val)?; // no need for mut ?! for
                                                   // shadowing here ?
            match fs::exists(&dir_path) {
                Ok(true) => builder.data_dir = Some(dir_path),
                Ok(false) => return Err(ConfigError::DataDirDoesNotExists),
                Err(e) => return Err(ConfigError::DataDirIoError(e)),
            }
        }

        Ok(builder)
    }

    /// Config builder from env. variables
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` variant
    pub fn from_config_file(cfg_path: &str) -> Result<Self, ConfigError> {
        use std::fs;
        let content = match fs::read_to_string(cfg_path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Warning: Failed to read config file '{cfg_path}': {err}");
                return Ok(Self::new());
            }
        };

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
                        "address" => builder.server_addr = Some(cfg_value.parse::<SocketAddr>()?),
                        "pool_size" => {
            let size = cfg_value.parse::<usize>()?;
            if size == 0 {
                return Err(ConfigError::PoolSizeZero);
            }
            builder.pool_size = Some(size);

                        }
                        "data_dir" => {
            let dir_path = fs::canonicalize(cfg_value)?; // no need for mut ?! for
                                                   // shadowing here ?
            match fs::exists(&dir_path) {
                Ok(true) => builder.data_dir = Some(dir_path),
                Ok(false) => return Err(ConfigError::DataDirDoesNotExists),
                Err(e) => return Err(ConfigError::DataDirIoError(e)),
            }}
,
                        _ => eprintln!("Warning: unknown key-value pair found in con)fig file [server] section: {cfg_key} = {cfg_value}"),
                    }
                }
            }
        }

        Ok(builder)
    }

    #[must_use]
    pub fn merge(&self, other: &Builder) -> Builder {
        Builder {
            server_addr: self.server_addr.or(other.server_addr),
            pool_size: self.pool_size.or(other.pool_size), // NOTE: usize is Copy, no clone needed
            data_dir: self.data_dir.clone().or(other.data_dir.clone()),
        }
    }
}
