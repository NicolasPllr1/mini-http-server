use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

// TODO:
// 1. error handling basic : std::error::Error
// 2. refactor using combinator/ ? if using custom error

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum ContentEncoding {
    GZip,
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContentEncoding::GZip => write!(f, "gzip"),
        }
    }
}

impl std::str::FromStr for ContentEncoding {
    type Err = String;

    fn from_str(encoding_scheme_str: &str) -> Result<Self, Self::Err> {
        match encoding_scheme_str {
            "gzip" => Ok(ContentEncoding::GZip),
            _ => Err(format!(
                "Only gzip supported. Proposed encoding schemes received: {encoding_scheme_str}",
            )),
        }
    }
}

impl ContentEncoding {
    pub fn encode_body(self, body: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        match self {
            ContentEncoding::GZip => gzip_encode_body(body),
        }
    }
    pub fn from_header(hdr_val: &str) -> Option<ContentEncoding> {
        // In the header: either a single scheme or a list of schemes
        hdr_val
            .split(',')
            .map(str::trim)
            .filter_map(|s| s.parse::<ContentEncoding>().ok())
            .find(|encoding| *encoding == ContentEncoding::GZip)
    }
}
fn gzip_encode_body(body: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(body)?;
    encoder.finish()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_encoding_basic_parsing() {
        assert_eq!(
            "gzip".parse::<ContentEncoding>().unwrap(),
            ContentEncoding::GZip
        );
        assert!("not_gzip".parse::<ContentEncoding>().is_err());
    }

    #[test]
    fn test_encoding_parsing_from_header_value() {
        assert_eq!(
            ContentEncoding::from_header("gzip").unwrap(),
            ContentEncoding::GZip
        );

        assert_eq!(
            ContentEncoding::from_header("deflate, gzip").unwrap(),
            ContentEncoding::GZip
        );

        assert_eq!(
            ContentEncoding::from_header("br, lorem, gzip, ipsum").unwrap(),
            ContentEncoding::GZip
        );

        assert!(ContentEncoding::from_header("br, lorem, ipsum").is_none());
        assert!("".parse::<ContentEncoding>().is_err());

        assert!(ContentEncoding::from_header("deflate").is_none());
    }
}
