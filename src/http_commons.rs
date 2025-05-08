#[derive(Debug, Copy, Clone)]
pub enum HttpVersion {
    Http11,
    Http2,
    // Http3,
}

#[derive(Debug)]
pub struct HttpVersionParseError {
    pub found: String,
}

impl std::str::FromStr for HttpVersion {
    type Err = HttpVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Self::Http11),
            "HTTP/2" => Ok(Self::Http2),
            _ => Err(HttpVersionParseError { found: s.into() }),
        }
    }
}

impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpVersion::Http11 => write!(f, "HTTP/1.1"),
            HttpVersion::Http2 => write!(f, "HTTP/2"),
        }
    }
}
