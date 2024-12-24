#![allow(dead_code)]
use std::io::prelude::*;
use std::net::TcpStream;

use std::io::BufReader;

use crate::html_commons::{Header, HttpVersion};

#[derive(Debug)]
pub struct HttpRequest {
    pub http_method: HttpMethod,
    pub ressource_path: String,
    pub protocol_version: HttpVersion,
    pub headers: Vec<Header>,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
}

impl HttpRequest {
    pub fn new_from_stream(stream: &mut TcpStream) -> HttpRequest {
        let reader = BufReader::new(stream);

        let html_request: Vec<String> = reader
            .lines()
            .map(|line_res| line_res.unwrap())
            .take_while(|line| !is_html_request_last_line(line))
            .collect();
        dbg!(&html_request);

        let (http_method, ressource_path, protocol_version) = match html_request[0]
            .split(" ")
            .collect::<Vec<&str>>()[..]
        {
            [m, p, v] => (m, p.to_string(), v),
            _ => panic!("Invalid HTTP request line: must have exactly method, path, and version"),
        };

        let http_method = match http_method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => panic!("Unsupported HTTP method"), // or return an error
        };

        dbg!(protocol_version);
        let protocol_version = match protocol_version {
            "HTTP/1.1" => HttpVersion::Http1,
            "HTTP/2" => HttpVersion::Http2,
            _ => panic!("Not HTTP/1.1 nor HTTP/2"),
        };

        HttpRequest {
            http_method,
            ressource_path,
            protocol_version,
            headers: Vec::new(),
        }
    }
}

fn is_html_request_last_line(line: &str) -> bool {
    let double_crfl = "\r\n\r\n"; // end-pattern : double CRFL
    if line.ends_with(double_crfl) {
        println!("Found the double CRFL !");
        true
    } else if line.is_empty() {
        println!("Found empty line : {}", line);
        true
    } else {
        println!("Not the last line : {}", line);
        false
    }
}
