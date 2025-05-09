use crate::encoding::ContentEncoding;
use crate::endpoints::{EndpointError, Endpoints};
use crate::http_commons::HttpVersion;
use crate::http_request::{HttpRequest, RequestError};
use bytes::Bytes;

use std::fmt::{self, Display};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

//TODO:
// 1. use combinator to reduce explicit matching
// 2. custom error type?

#[derive(Debug)]
pub struct HttpResponse {
    pub protocol_version: HttpVersion,
    pub status_code: StatusCode,
    pub content_type: ContentType,
    pub content_length: usize,
    pub content_encoding: Option<ContentEncoding>,
    pub conn_close: bool,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    Ok,
    NotFound,
    Created,
    NotImplemented,
    InternalServerError,
    BadRequest,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StatusCode::Ok => write!(f, "200 OK"),
            StatusCode::NotFound => write!(f, "404 Not Found"),
            StatusCode::Created => write!(f, "201 Created"),
            StatusCode::NotImplemented => write!(f, "501 Not Implemented"),
            StatusCode::InternalServerError => write!(f, "500 Internal Server Error"),
            StatusCode::BadRequest => write!(f, "400 Bad Request"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    Html,
    Css,
    JavaScript,
    Json,
    Png,
    Jpeg,
    Gif,
    Svg,
    PlainText,
    Pdf,
    OctetStream, // default
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(ext: &str) -> Result<Self, Self::Err> {
        Ok(match ext {
            "html" | "htm" => ContentType::Html,
            "css" => ContentType::Css,
            "js" => ContentType::JavaScript,
            "json" => ContentType::Json,
            "png" => ContentType::Png,
            "jpg" | "jpeg" => ContentType::Jpeg,
            "gif" => ContentType::Gif,
            "svg" => ContentType::Svg,
            "txt" => ContentType::PlainText,
            "pdf" => ContentType::Pdf,
            _ => ContentType::OctetStream,
        })
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mime = match self {
            ContentType::Html => "text/html",
            ContentType::Css => "text/css",
            ContentType::JavaScript => "application/javascript",
            ContentType::Json => "application/json",
            ContentType::Png => "image/png",
            ContentType::Jpeg => "image/jpeg",
            ContentType::Gif => "image/gif",
            ContentType::Svg => "image/svg+xml",
            ContentType::PlainText => "text/plain",
            ContentType::Pdf => "application/pdf",
            ContentType::OctetStream => "application/octet-stream",
        };
        write!(f, "{mime}")
    }
}

#[derive(Debug)]
pub enum ResponseError {
    Endpoint(EndpointError),
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseError::Endpoint(e) => write!(f, "error processing the endpoint: {e}"),
        }
    }
}

impl std::error::Error for ResponseError {}

impl From<EndpointError> for ResponseError {
    fn from(e: EndpointError) -> ResponseError {
        ResponseError::Endpoint(e)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct HttpResponseBuilder {
    http_response: HttpResponse,
}

pub trait Builder<T> {
    fn new() -> Self;
    fn build(self) -> T;
}

impl Builder<HttpResponse> for HttpResponseBuilder {
    fn new() -> Self {
        // response with default values
        Self {
            http_response: HttpResponse {
                protocol_version: HttpVersion::Http11,
                status_code: StatusCode::Ok,
                content_type: ContentType::PlainText,
                content_length: 0,
                content_encoding: None,
                conn_close: false,
                body: None,
            },
        }
    }
    fn build(self) -> HttpResponse {
        self.http_response
    }
}

#[allow(dead_code)]
impl HttpResponseBuilder {
    pub fn with_protocol_version(&mut self, protocol_version: HttpVersion) {
        self.http_response.protocol_version = protocol_version;
    }

    pub fn with_status_code(&mut self, status_code: StatusCode) {
        self.http_response.status_code = status_code;
    }

    pub fn with_content_type(&mut self, content_type: ContentType) {
        self.http_response.content_type = content_type;
    }

    pub fn with_content_length(&mut self, content_length: usize) {
        self.http_response.content_length = content_length;
    }

    pub fn with_content_encoding(&mut self, content_encoding: Option<ContentEncoding>) {
        self.http_response.content_encoding = content_encoding;
    }

    pub fn with_conn_close(&mut self, conn_close: bool) {
        self.http_response.conn_close = conn_close;
    }

    pub fn with_body(&mut self, body: &str) {
        self.http_response.body = Some(body.to_string());
    }
}

pub trait Buildable<Target, B: Builder<Target>> {
    // NOTE: gives you an instance of the Builder from the Target
    // (Target being the type implementing this trait, i.e. being "Buildable")
    fn builder() -> B;
}

impl Buildable<HttpResponse, HttpResponseBuilder> for HttpResponse {
    fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::new()
    }
}

// Public API
impl HttpResponse {
    /// Builds a HTTP response based on an HTTP request
    /// # Errors
    /// Endpoints can return errors.
    pub fn new_from_request(http_request: &HttpRequest, data_dir: &Path) -> HttpResponse {
        if let Ok(endpoint_requested) = &http_request.request_target.parse::<Endpoints>() {
            if let Ok(response) = endpoint_requested.handle_request(http_request, data_dir) {
                response
            } else {
                let mut builder = HttpResponse::builder();
                builder.with_status_code(StatusCode::NotImplemented);
                builder.build()
            }
        } else {
            let mut builder = HttpResponse::builder();
            builder.with_status_code(StatusCode::InternalServerError);
            builder.build()
        }
    }
    pub fn new_from_bad_request(error: &RequestError) -> HttpResponse {
        let mut builder = HttpResponse::builder();

        builder.with_status_code(StatusCode::BadRequest);

        let body = error.to_string();
        builder.with_body(&body);
        builder.with_content_length(body.len());

        builder.build()
    }

    /// Write HTTP response
    /// # Errors
    /// Some write steps may return an error.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Status line
        write!(writer, "{} {}\r\n", self.protocol_version, self.status_code)?;

        // Content-type
        write!(writer, "Content-Type: {}\r\n", self.content_type)?;

        // Close connection
        if self.conn_close {
            write!(writer, "Connection: close\r\n")?;
        }

        // Body if any
        if let Some(body) = &self.body {
            let encoded_body_bytes = if let Some(encoding) = &self.content_encoding {
                write!(writer, "{encoding}")?;
                encoding.encode_body(body)
            } else {
                Bytes::from(body.as_bytes().to_vec())
            };

            write!(
                writer,
                "Content-Length: {}\r\n\r\n",
                encoded_body_bytes.len()
            )?;
            writer.write_all(&encoded_body_bytes)?;
            // write!(writer, "\r\n")?;
        } else {
            // no body, the end
            write!(writer, "\r\n")?;
        }
        Ok(())
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Status line
        write!(f, "{} {}\r\n", self.protocol_version, self.status_code,)?;

        // Content-type
        write!(f, "Content-Type: {}\r\n", self.content_type)?;

        // Close connection
        if self.conn_close {
            write!(f, "Connection: close\r\n")?;
        }

        // Body if any
        if let Some(body) = &self.body {
            // TODO: why borrowing self.body is needed here ? same
            // below for contentncoding
            if let Some(encoding) = &self.content_encoding {
                write!(f, "{encoding}")?;
            }
            write!(f, "Content-Length: {}", body.len())?; // TODO: why putting self.body.len() does
                                                          // not work ? ('fn is private')
            write!(f, "{body}")?;
        }

        // write!(f, "\r\n")?;
        Ok(())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_commons::HttpVersion;
    use crate::http_request::HttpMethod;
    use std::collections::HashMap;

    use crate::encoding::ContentEncoding;

    fn create_test_request(path: &str) -> HttpRequest {
        HttpRequest {
            http_method: HttpMethod::Get,
            request_target: path.to_string(),
            protocol_version: HttpVersion::Http11,
            headers: HashMap::new(),
            body: None,
        }
    }

    #[test]
    fn test_echo_endpoint_basic() {
        let request = create_test_request("/echo/hello");
        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 5);
        assert_eq!(response.body.unwrap(), "hello");
    }

    #[test]
    fn test_echo_endpoint_empty() {
        let request = create_test_request("/echo/");
        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 0);
        assert_eq!(response.body.unwrap(), "");
    }

    #[test]
    fn test_echo_endpoint_with_spaces() {
        let request = create_test_request("/echo/hello world");
        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 11);
        assert_eq!(response.body.unwrap(), "hello world");
    }

    #[test]
    fn test_echo_endpoint_special_chars() {
        let request = create_test_request("/echo/hello!@#$%");
        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, 10);
        assert_eq!(response.body.unwrap(), "hello!@#$%");
    }

    #[test]
    fn test_response_write_to() {
        let request = create_test_request("/echo/test");
        let response = HttpResponse::new_from_request(&request, &Path::new(""));
        let mut output = Vec::new();

        response.write_to(&mut output).unwrap();

        let response_str = String::from_utf8(output).unwrap();
        assert!(response_str.contains("200 OK"));
        assert!(response_str.contains("Content-Type: text/plain"));
        assert!(response_str.contains("Content-Length: 4"));
        assert!(response_str.contains("test"));
    }

    #[test]
    fn test_echo_endpoint_with_gzip_encoding() {
        let mut request = create_test_request("/echo/compressed_content");
        request
            .headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_encoding, Some(ContentEncoding::GZip));

        assert!(response.content_length > 0);

        // NOTE: body gets compressed when response is written as bytes
        // --> the body should not be compressed yet
        assert_eq!(response.body.as_ref().unwrap(), "compressed_content");
    }

    #[test]
    fn test_echo_endpoint_with_multiple_encodings() {
        let mut request = create_test_request("/echo/hello");
        request.headers.insert(
            "Accept-Encoding".to_string(),
            "deflate, gzip, br".to_string(),
        );

        let response = HttpResponse::new_from_request(&request, Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        // Should choose gzip as it's supported and within the list of proposed encoding schemes
        assert_eq!(response.content_encoding, Some(ContentEncoding::GZip));
    }

    #[test]
    fn test_echo_endpoint_with_unsupported_encoding() {
        let mut request = create_test_request("/echo/hello");
        request
            .headers
            .insert("Accept-Encoding".to_string(), "deflate, br".to_string());

        let response = HttpResponse::new_from_request(&request, &Path::new(""));

        assert!(matches!(response.status_code, StatusCode::Ok));
        // Should not have Content-Encoding header as no supported encoding was requested
        assert!(response.content_encoding.is_none());
        assert_eq!(response.body.unwrap(), "hello");
    }

    #[test]
    fn test_response_write_to_with_gzip() {
        let mut request = create_test_request("/echo/test_compressed");
        request
            .headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let response = HttpResponse::new_from_request(&request, &Path::new(""));
        let mut rcv_buff = Vec::new();
        response.write_to(&mut rcv_buff).unwrap();

        // Only check the headers portion which is not compressed
        let headers_str = String::from_utf8_lossy(&rcv_buff);
        assert!(headers_str.contains("200 OK"));
        assert!(headers_str.contains("Content-Type: text/plain"));
        assert!(headers_str.contains("Content-Encoding: gzip"));
    }
}
