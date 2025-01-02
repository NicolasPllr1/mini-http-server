use bytes::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

#[derive(Debug)]
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

impl ContentEncoding {
    pub fn encode_body(&self, body: &str) -> Bytes {
        match self {
            ContentEncoding::GZip => gzip_encode_body(body).unwrap_or_default(),
        }
    }
}
pub fn gzip_encode_body(body: &str) -> Option<Bytes> {
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
