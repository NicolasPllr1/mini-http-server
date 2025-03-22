use bytes::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

// TODO:
// 1. error handling basic : std::error::Error
// 2. refactor using combinator/ ? if using custom error

#[derive(Debug, PartialEq)]
pub enum ContentEncoding {
    GZip,
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContentEncoding::GZip => write!(f, "Content-Encoding: gzip"),
        }
    }
}

impl std::str::FromStr for ContentEncoding {
    type Err = String;

    fn from_str(encodings_list_str: &str) -> Result<Self, Self::Err> {
        // encoding_schemes : either a single value OR a list of values
        let encoding_scheme: Option<ContentEncoding> = encodings_list_str
            .split(",")
            .map(|s| s.trim())
            .filter_map(|s| s.parse::<ContentEncoding>().ok())
            .find(|encoding| *encoding == ContentEncoding::GZip);
        encoding_scheme.ok_or(format!(
            "Only gzip supported. Proposed encoding schemes received: {}",
            encodings_list_str
        ))
    }
}

impl ContentEncoding {
    pub fn encode_body(&self, body: &str) -> Bytes {
        match self {
            ContentEncoding::GZip => gzip_encode_body(body).unwrap_or_default(),
        }
    }
    pub fn parse_encoding_scheme(encoding_str: &str) -> Option<ContentEncoding> {
        // encoding_schemes : either a single value OR a list of values
        let encoding_scheme: Option<ContentEncoding> = encoding_str
            .split(",")
            .map(|s| s.trim())
            .filter_map(|s| s.parse::<ContentEncoding>().ok())
            .find(|encoding| *encoding == ContentEncoding::GZip);
        encoding_scheme
    }
}
fn gzip_encode_body(body: &str) -> Option<Bytes> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    println!("Body to encode: {}", &body);
    match encoder.write_all(body.as_bytes()) {
        Ok(_) => {
            println!("Successful gzip compression started");
        }
        Err(e) => {
            println!("Error while initiating gzip-compressing the body: {}", e);
            println!("Returning None");
            return None;
        }
    };
    let compressed_body = match encoder.finish() {
        Ok(encoded_bytes) => {
            println!("Gzip compression successfull");
            Some(Bytes::from(encoded_bytes))
        }
        Err(e) => {
            println!("Error during the gzip-compression : {}", e);
            None
        }
    };
    compressed_body
}
