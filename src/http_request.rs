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

impl std::str::FromStr for HttpMethod {
    type Err = String; // NOTE: what/why ?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            _ => Err("Unsupported HTTP method".into()),
        }
    }
}

impl HttpRequest {
    pub fn build_from_stream(stream: &mut TcpStream) -> Result<HttpRequest, Box<dyn Error>> {
        let mut reader = BufReader::new(&mut *stream);

        // Read the *request-line*
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;
        println!("Sucess reading the *request-line*: {}", request_line);

        // Parse the *request-line*
        let [http_method, request_target, protocol_version]: [&str; 3] = request_line
            .split_whitespace()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|e| format!("Invalid HTTP request-line. Expected: <method> <request-target> <protocol-version>. Got: {:?}", e))?;

        let request_target = request_target.to_string();

        let http_method = http_method.parse::<HttpMethod>()?; // turbofish yeah + shadowing

        let protocol_version = protocol_version.parse::<HttpVersion>()?;

        // Read eventual *headers*
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut content = String::new(); // NOTE: could allocate once?
            reader.read_line(&mut content).unwrap();
            if content == "\r\n" {
                break;
            } else {
                let (header_name, header_value) =
                    content.split_once(":").expect("Invalid header section");
                headers.insert(header_name.to_string(), header_value.trim().to_string());
            }
        }

        // Read the *body* if any
        let body = if let Some(n_bytes_str) = headers.get("Content-Length") {
            let n_bytes = n_bytes_str
                .parse::<usize>()
                .map_err(|e| format!("Invalid content-length (for body): {}", e))?;

            let mut body_buf = vec![0; n_bytes];
            reader.read_exact(&mut body_buf)?;

            Some(
                String::from_utf8(body_buf)
                    .map_err(|e| format!("Invalid utf-8 for body content: {}", e))?,
            )
        } else {
            None
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
