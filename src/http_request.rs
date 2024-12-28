#![allow(dead_code)]
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
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
}

impl HttpRequest {
    pub fn new_from_stream(stream: &mut TcpStream) -> HttpRequest {
        let reader = BufReader::new(stream);

        let request_parts: Vec<String> = reader
            .lines()
            .map(|line_res| line_res.unwrap())
            .take_while(|line| !is_html_request_last_line(line))
            .collect();
        // dbg!(&request_parts);

        // Parsing for the *request-line*
        let request_line = &request_parts[0];
        let (http_method, request_target, protocol_version) =
            match request_line.split(" ").collect::<Vec<&str>>()[..] {
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

        let protocol_version = match protocol_version {
            "HTTP/1.1" => HttpVersion::Http1,
            "HTTP/2" => HttpVersion::Http2,
            _ => panic!("Not HTTP/1.1 nor HTTP/2"),
        };

        // Parsing for *headers*
        let mut headers: HashMap<String, String> = HashMap::new();
        for content in &request_parts[1..] {
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

        HttpRequest {
            http_method,
            request_target,
            protocol_version,
            headers,
        }
    }
}

fn is_html_request_last_line(line: &str) -> bool {
    let double_crfl = "\r\n\r\n"; // end-pattern : double CRFL
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
