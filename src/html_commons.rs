#[derive(Debug, Copy, Clone)]
pub enum HttpVersion {
    Http1,
    Http2,
    // Http3,
}
impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpVersion::Http1 => write!(f, "HTTP/1.1"),
            HttpVersion::Http2 => write!(f, "HTTP/2"),
        }
    }
}
#[derive(Debug)]
pub struct Header {
    name: String,
    value: String,
}
