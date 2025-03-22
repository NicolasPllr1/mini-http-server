use crate::encoding::ContentEncoding;
use crate::endpoints::Endpoints;
use crate::http_commons::HttpVersion;
use crate::http_request::HttpRequest;

use std::error::Error;
use std::fmt::Display;
use std::io::Write;

//TODO:
// 1. use combinator to reduce explicit matching
// 2. custom error with ?

#[derive(Debug)]
pub struct HttpResponse {
    pub protocol_version: HttpVersion,
    pub status_code: StatusCode,
    pub content_type: String,
    pub content_length: usize,
    pub content_encoding: Option<ContentEncoding>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum StatusCode {
    Ok,
    NotFound,
    Created,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StatusCode::Ok => write!(f, "200 OK"),
            StatusCode::NotFound => write!(f, "404 Not Found"),
            StatusCode::Created => write!(f, "201 Created"),
        }
    }
}

// Public API
impl HttpResponse {
    pub fn build_from_request(
        http_request: &HttpRequest,
        data_dir: &str,
    ) -> Result<HttpResponse, Box<dyn Error>> {
        let endpoint_requested = &http_request.request_target.parse::<Endpoints>()?;
        println!("endpoint requested: {:?}", endpoint_requested);
        endpoint_requested.handle_request(http_request, data_dir)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.content_length == 0 {
            write!(
                writer,
                "{} {}\r\n\r\n",
                self.protocol_version, self.status_code
            )
        } else {
            match &self.content_encoding {
                Some(encoding_scheme) => {
                    let encoded_body_bytes =
                        encoding_scheme.encode_body(&self.body.as_deref().unwrap_or_default());
                    // First write *headers*
                    write!(
                        writer,
                        "{} {}\r\ncontent-type: {}\r\n{}\r\ncontent-length: {}\r\n\r\n",
                        self.protocol_version,
                        self.status_code,
                        self.content_type,
                        encoding_scheme,
                        encoded_body_bytes.len(),
                    )?;
                    // Second write the *encoded-body* (raw bytes directly)
                    writer.write_all(&encoded_body_bytes)
                }
                None => {
                    write!(
                        writer,
                        "{} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\n\r\n{}",
                        self.protocol_version,
                        self.status_code,
                        self.content_type,
                        self.content_length,
                        self.body.clone().unwrap_or_default(),
                    )
                }
            }
        }
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content_length == 0 {
            write!(f, "{} {}\r\n\r\n", self.protocol_version, self.status_code)
        } else {
            match &self.content_encoding {
                Some(encoding_scheme) => {
                    let encoded_body_bytes =
                        encoding_scheme.encode_body(&self.body.as_deref().unwrap_or_default());
                    let encoded_body_hexa = encoded_body_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b).to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    write!(
                        f,
                        "{} {}\r\ncontent-type: {}\r\n{}\r\ncontent-length: {}\r\n\r\n{}",
                        self.protocol_version,
                        self.status_code,
                        self.content_type,
                        encoding_scheme,
                        encoded_body_bytes.len(),
                        encoded_body_hexa,
                    )
                }
                None => {
                    write!(
                        f,
                        "{} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\n\r\n{}",
                        self.protocol_version,
                        self.status_code,
                        self.content_type,
                        self.content_length,
                        self.body.clone().unwrap_or_default(),
                    )
                }
            }
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_commons::HttpVersion;
    use crate::http_request::HttpMethod;
    use std::collections::HashMap;

    fn create_test_request(path: &str) -> HttpRequest {
        HttpRequest {
            http_method: HttpMethod::GET,
            request_target: path.to_string(),
            protocol_version: HttpVersion::Http11,
            headers: HashMap::new(),
            body: None,
        }
    }

    #[test]
    fn test_echo_endpoint_basic() {
        let request = create_test_request("/echo/hello");
        let response = HttpResponse::build_from_request(&request, "").unwrap();

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 5);
        assert_eq!(response.body.unwrap(), "hello");
    }

    #[test]
    fn test_echo_endpoint_empty() {
        let request = create_test_request("/echo/");
        let response = HttpResponse::build_from_request(&request, "").unwrap();

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 0);
        assert_eq!(response.body.unwrap(), "");
    }

    #[test]
    fn test_echo_endpoint_with_spaces() {
        let request = create_test_request("/echo/hello world");
        let response = HttpResponse::build_from_request(&request, "").unwrap();

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 11);
        assert_eq!(response.body.unwrap(), "hello world");
    }

    #[test]
    fn test_echo_endpoint_special_chars() {
        let request = create_test_request("/echo/hello!@#$%");
        let response = HttpResponse::build_from_request(&request, "").unwrap();

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 10);
        assert_eq!(response.body.unwrap(), "hello!@#$%");
    }

    #[test]
    fn test_response_write_to() {
        let request = create_test_request("/echo/test");
        let response = HttpResponse::build_from_request(&request, "").unwrap();
        let mut output = Vec::new();

        response.write_to(&mut output).unwrap();

        let response_str = String::from_utf8(output).unwrap();
        assert!(response_str.contains("200 OK"));
        assert!(response_str.contains("content-type: text/plain"));
        assert!(response_str.contains("content-length: 4"));
        assert!(response_str.contains("test"));
    }
}
