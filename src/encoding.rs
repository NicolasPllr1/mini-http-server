use bytes::Bytes;
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
            ContentEncoding::GZip => write!(f, "Content-Encoding: gzip\r\n"),
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
    #[must_use]
    pub fn encode_body(self, body: &str) -> Bytes {
        match self {
            ContentEncoding::GZip => gzip_encode_body(body).unwrap_or_default(),
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
fn gzip_encode_body(body: &str) -> Option<Bytes> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    println!("Body to encode: {}", &body);
    match encoder.write_all(body.as_bytes()) {
        Ok(()) => {
            println!("gzip compression initiated");
        }
        Err(e) => {
            println!("Error while initiating gzip-compressing the body: {e}");
            println!("Returning None");
            return None;
        }
    };
    match encoder.finish() {
        Ok(encoded_bytes) => {
            println!("Gzip compression successful");
            Some(Bytes::from(encoded_bytes))
        }
        Err(e) => {
            println!("Error during the gzip-compression : {e}");
            None
        }
    }
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
