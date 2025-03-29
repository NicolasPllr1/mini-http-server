use crate::encoding::ContentEncoding;
use crate::endpoints::Endpoints;
use crate::http_commons::HttpVersion;
use crate::http_request::HttpRequest;
use bytes::Bytes;

use std::error::Error;
use std::fmt::Display;
use std::io::Write;

//TODO:
// 1. use combinator to reduce explicit matching
// 2. custom error type?

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
        endpoint_requested.handle_request(http_request, data_dir)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Status line
        write!(writer, "{} {}\r\n", self.protocol_version, self.status_code)?;

        // Content-type
        write!(writer, "Content-Type: {}\r\n", self.content_type)?;

        // Body if any
        if let Some(body) = &self.body {
            let encoded_body_bytes = if let Some(encoding) = &self.content_encoding {
                write!(writer, "{}", encoding)?;
                encoding.encode_body(body)
            } else {
                Bytes::from(body.as_bytes().to_vec())
            };

            write!(
                writer,
                "Content-Length: {}\r\n\r\n",
                encoded_body_bytes.len()
            )?;
            let _ = writer.write_all(&encoded_body_bytes)?;
            write!(writer, "\r\n")?;
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

        // Body if any
        if let Some(body) = &self.body {
            // TODO: why borrowing self.body is needed here ? same
            // below for content_encoding
            if let Some(encoding) = &self.content_encoding {
                write!(f, "{}", encoding)?;
            }
            write!(f, "Content-Length: {}", body.len())?; // TODO: why putting self.body.len() does
                                                          // not work ? ('fn is private')
            write!(f, "{}", body)?;
        }

        write!(f, "\r\n")?;
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

    #[test]
    fn test_echo_endpoint_with_gzip_encoding() {
        let mut request = create_test_request("/echo/compressed_content");
        request
            .headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let response = HttpResponse::build_from_request(&request, "").unwrap();

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

        let response = HttpResponse::build_from_request(&request, "").unwrap();

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

        let response = HttpResponse::build_from_request(&request, "").unwrap();

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

        let response = HttpResponse::build_from_request(&request, "").unwrap();
        let mut rcv_buff = Vec::new();
        response.write_to(&mut rcv_buff).unwrap();

        // Only check the headers portion which is not compressed
        let headers_str = String::from_utf8_lossy(&rcv_buff);
        assert!(headers_str.contains("200 OK"));
        assert!(headers_str.contains("content-type: text/plain"));
        assert!(headers_str.contains("Content-Encoding: gzip"));
    }
}
