#![allow(dead_code)]
use crate::http_commons::{HttpVersion, HttpVersionParseError};

use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct HttpRequest {
    pub http_method: HttpMethod,
    pub request_target: String,
    pub protocol_version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum RequestError {
    Io(std::io::Error),
    RequestLine(String),
    Method(String),
    ProtocolVersion(String),
    Header(String),
    BodyUtf8(std::string::FromUtf8Error),
    BodyContentLength(ParseIntError),
}

impl From<std::io::Error> for RequestError {
    fn from(e: std::io::Error) -> RequestError {
        RequestError::Io(e)
    }
}

impl From<HttpMethodParseError> for RequestError {
    fn from(e: HttpMethodParseError) -> RequestError {
        RequestError::Method(e.found)
    }
}
impl From<HttpVersionParseError> for RequestError {
    fn from(e: HttpVersionParseError) -> RequestError {
        RequestError::ProtocolVersion(e.found)
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::RequestLine(s) => write!(f, "malformed request line: {s}"),
            RequestError::Method(m) => write!(f, "unsupported HTTP method: {m}"),
            RequestError::ProtocolVersion(v) => write!(f, "unsupported HTTP protocol version: {v}"),
            RequestError::Header(h) => write!(f, "invalid header: {h}"),
            RequestError::BodyUtf8(b) => write!(f, "body is not valid UTF-8: {b}"),
            RequestError::BodyContentLength(l) => {
                write!(f, "error parsing the body length: {l}")
            }
            RequestError::Io(e) => write!(f, "I/O while reading request: {e}"),
        }
    }
}
impl std::error::Error for RequestError {}

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
                http_method: HttpMethod::Get,
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
        self.http_request.headers.clone_from(headers);
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
    /// Builds a HTTP request from a parsing an incoming stream of bytes, that should
    /// corresponds to a valid HTTP request.
    /// # Errors
    /// Returns a `RequestError` variant
    pub fn build_from_stream(stream: &mut TcpStream) -> Result<HttpRequest, RequestError> {
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
            .map_err(|_| RequestError::RequestLine(request_line.to_string()))?;
        // .map_err(|e| format!("Invalid HTTP request-line. Expected: <method> <request-target> <protocol-version>. Got: {e:?}"))?;

        let http_method = http_method.parse::<HttpMethod>()?; // turbofish yeah + shadowing + ?
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
            let (header_name, header_value) = content
                .split_once(':')
                .ok_or(RequestError::Header(content.to_string()))?;
            headers.insert(header_name.to_string(), header_value.trim().to_string());
        }

        builder.with_headers(&headers);

        // Read the *body* if any
        if let Some(n_bytes_str) = headers.get("Content-Length") {
            let n_bytes = n_bytes_str
                .parse::<usize>()
                .map_err(RequestError::BodyContentLength)?;

            let mut body_buf = vec![0; n_bytes];
            reader.read_exact(&mut body_buf)?;

            let body = String::from_utf8(body_buf).map_err(RequestError::BodyUtf8)?;

            builder.with_body(&body);
        };

        Ok(builder.build())
    }
    pub fn keep_alive(&self) -> bool {
        match self.headers.get("Connection") {
            Some(s) if s == "close" => false,
            Some(_) | None => true,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug)]
pub struct HttpMethodParseError {
    pub found: String,
}

impl std::str::FromStr for HttpMethod {
    type Err = HttpMethodParseError; // NOTE: what/why ?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(HttpMethodParseError { found: s.into() }),
        }
    }
}
