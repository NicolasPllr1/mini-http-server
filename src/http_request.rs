use std::io::prelude::*;
use std::net::TcpStream;

use std::io::BufReader;

use crate::http_commons::HttpVersion;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub http_method: HttpMethod,
    pub request_target: String,
    pub protocol_version: HttpVersion,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
}

impl HttpRequest {
    pub fn new_from_stream(stream: &mut TcpStream) -> HttpRequest {
        let mut reader = BufReader::new(&mut *stream);
        // let request_parts: Vec<String> = reader
        //     .lines()
        //     .map(|line_res| line_res.unwrap())
        //     .take_while(|line| !is_html_request_last_line(line))
        //     .collect();
        // dbg!(&request_parts);

        // Read the *request-line*
        let mut request_line = String::new();
        match reader.read_line(&mut request_line) {
            Ok(_) => println!("Sucess reading the *request-line*: {}", request_line),
            Err(e) => panic!("Error reading the *request-line*: {}", e),
        };

        // Parse the *request-line*
        let (http_method, request_target, protocol_version) =
            match request_line.split_whitespace().collect::<Vec<&str>>()[..] {
                [m, p, v] => (m, p.to_string(), v),
                _ => panic!(
                "Invalid HTTP request-line. Expected: <method> <request-target> <protocol-version>"
            ),
            };

        let http_method = match http_method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => panic!("Unsupported HTTP method"),
        };
        dbg!(&http_method);

        let protocol_version = match protocol_version {
            "HTTP/1.1" => HttpVersion::Http1,
            "HTTP/2" => HttpVersion::Http2,
            _ => panic!("Not HTTP/1.1 nor HTTP/2. Got: {}", protocol_version),
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
        // Wrap headers in an Option to signal their presence/abscence in the request
        let headers = if headers.is_empty() {
            None
        } else {
            Some(headers)
        };
        dbg!(&headers);

        // Read the body if any
        let body: Option<String> = match headers {
            Some(ref headers) => match headers.get("Content-Length") {
                Some(n_bytes_str) => {
                    let n_bytes = n_bytes_str
                        .parse::<usize>()
                        .expect("Error parsing the content length");
                    dbg!(n_bytes);
                    let mut body_buf = vec![0; n_bytes];
                    reader
                        .read_exact(&mut body_buf)
                        .expect("Could not read body into buffer");
                    Some(String::from_utf8(body_buf).expect("Invalid utf-8 for body"))
                }
                None => None,
            },
            None => None,
        };
        dbg!(&body);

        HttpRequest {
            http_method,
            request_target,
            protocol_version,
            headers,
            body,
        }
    }
}

fn is_html_request_last_line(line: &str) -> bool {
    let double_crfl = "\r\n\r\n"; // end-pattern : double CRFL
                                  // dbg!(&line);
    if line.ends_with(double_crfl) {
        // println!("Found the double CRFL !");
        true
    } else if line.is_empty() {
        // println!("Found empty line : {}", line);
        true
    } else {
        // println!("Not the last line : {}", line);
        false
    }
}
