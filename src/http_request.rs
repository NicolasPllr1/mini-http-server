#![allow(dead_code)]
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

#[allow(clippy::module_name_repetitions)]
pub struct HttpRequestBuilder {
    http_request: HttpRequest,
}

trait Builder<T> {
    fn new() -> Self;
    fn build(self) -> T;
}

impl Builder<HttpRequest> for HttpRequestBuilder {
    fn new() -> Self {
        // request with default values
        Self {
            http_request: HttpRequest {
                http_method: HttpMethod::GET,
                request_target: String::new(),
                protocol_version: HttpVersion::Http11,
                headers: HashMap::new(),
                body: None,
            },
        }
    }
    fn build(self) -> HttpRequest {
        self.http_request
    }
}

impl HttpRequestBuilder {
    // TODO: request-line is mandatory: method, target & protocol-version --> group them?
    fn with_method(&mut self, method: HttpMethod) {
        self.http_request.http_method = method;
    }

    // NOTE: could automate this through a macro
    fn with_target(&mut self, request_target: &str) {
        self.http_request.request_target = request_target.to_string();
    }
    fn with_protocol_version(&mut self, protocol_version: HttpVersion) {
        self.http_request.protocol_version = protocol_version;
    }
    fn with_headers(&mut self, headers: &HashMap<String, String>) {
        // TODO: possible to get rid of this clone ? How bad is this, design&perf wise ?
        self.http_request.headers = headers.clone();
    }
    fn with_body(&mut self, body: &str) {
        self.http_request.body = Some(body.to_string());
    }
}

trait Buildable<Target, B: Builder<Target>> {
    // NOTE: gives you an instance of the Builder from the Target
    // (Target being the type implementing this trait, i.e. being "Buildable")
    fn builder() -> B;
}

impl Buildable<HttpRequest, HttpRequestBuilder> for HttpRequest {
    fn builder() -> HttpRequestBuilder {
        HttpRequestBuilder::new()
    }
}

impl HttpRequest {
    pub fn build_from_stream(stream: &mut TcpStream) -> Result<HttpRequest, Box<dyn Error>> {
        let mut builder = HttpRequest::builder();
        let mut reader = BufReader::new(&mut *stream);

        // Read the *request-line*
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;
        println!("Success reading the *request-line*: {request_line}");

        // Parse the *request-line*
        let [http_method, request_target, protocol_version]: [&str; 3] = request_line
            .split_whitespace()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|e| format!("Invalid HTTP request-line. Expected: <method> <request-target> <protocol-version>. Got: {e:?}"))?;

        let http_method = http_method.parse::<HttpMethod>()?; // turbofish yeah + shadowing
        let protocol_version = protocol_version.parse::<HttpVersion>()?;

        builder.with_method(http_method);
        builder.with_target(request_target);
        builder.with_protocol_version(protocol_version);

        // Read eventual *headers*
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut content = String::new(); // TODO: allocate once before looping?
            reader.read_line(&mut content)?;
            if content == "\r\n" {
                break;
            }
            let (header_name, header_value) =
                content.split_once(':').ok_or("Invalid header section")?;
            headers.insert(header_name.to_string(), header_value.trim().to_string());
        }

        builder.with_headers(&headers);

        // Read the *body* if any
        if let Some(n_bytes_str) = headers.get("Content-Length") {
            let n_bytes = n_bytes_str
                .parse::<usize>()
                .map_err(|e| format!("Invalid content-length (for body): {e}"))?;

            let mut body_buf = vec![0; n_bytes];
            reader.read_exact(&mut body_buf)?;

            let body = String::from_utf8(body_buf)
                .map_err(|e| format!("Invalid utf-8 for body content: {e}"))?;

            builder.with_body(&body);
        };

        Ok(builder.build())
    }
}

#[derive(Debug, Clone, Copy)]
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
