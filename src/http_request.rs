use crate::http_commons::HttpVersion;

use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[derive(Debug)]
pub struct HttpRequest {
    pub http_method: HttpMethod,
    pub request_target: String,
    pub protocol_version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
}

impl HttpRequest {
    pub fn build_from_stream(stream: &mut TcpStream) -> Result<HttpRequest, Box<dyn Error>> {
        let mut reader = BufReader::new(&mut *stream);

        // Read the *request-line*
        let mut request_line = String::new();
        match reader.read_line(&mut request_line) {
            Ok(_) => println!("Sucess reading the *request-line*: {}", request_line),
            Err(e) => return Err(format!("Error reading the *request-line*: {}", e).into()),
        };

        // Parse the *request-line*
        let (http_method, request_target, protocol_version) = match request_line
            .split_whitespace()
            .collect::<Vec<&str>>()[..]
        {
            [m, p, v] => (m, p.to_string(), v),
            _ => return Err(
                "Invalid HTTP request-line. Expected: <method> <request-target> <protocol-version>"
                    .into(),
            ),
        };

        let http_method = match http_method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => return Err("Unsupported HTTP method".into()),
        };

        let protocol_version = match protocol_version {
            "HTTP/1.1" => HttpVersion::Http11,
            "HTTP/2" => HttpVersion::Http2,
            _ => return Err(format!("Not HTTP/1.1 nor HTTP/2. Got: {}", protocol_version).into()),
        };

        // Read eventual *headers*
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut content = String::new();
            reader.read_line(&mut content).unwrap();
            if content == "\r\n" {
                break;
            } else {
                let (header_name, header_value) =
                    content.split_once(":").expect("Invalid header section");
                headers.insert(header_name.to_string(), header_value.trim().to_string());
            }
        }

        // Read the body if any
        let body: Option<String> = match headers.get("Content-Length") {
            Some(n_bytes_str) => {
                let n_bytes = n_bytes_str
                    .parse::<usize>()
                    .expect("Error parsing the content length");
                let mut body_buf = vec![0; n_bytes];
                reader
                    .read_exact(&mut body_buf)
                    .expect("Could not read body into buffer");
                Some(String::from_utf8(body_buf).expect("Invalid utf-8 for body"))
            }
            None => None,
        };

        Ok(HttpRequest {
            http_method,
            request_target,
            protocol_version,
            headers,
            body,
        })
    }
}
