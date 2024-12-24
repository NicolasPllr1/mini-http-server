// <protocol> <status-code> <status-text>
// e.g. HTTP/1.1 200 OK <status-text>

use crate::html_commons::{Header, HttpVersion};
use crate::html_request::HttpRequest;

pub struct HttpResponse {
    protocol_version: HttpVersion,
    status_code: StatusCode,
    content_type: String,
    content_length: usize,
    headers: Vec<Header>,
    body: String,
}

#[derive(Debug)]
enum StatusCode {
    Ok,
    NotFound,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StatusCode::Ok => write!(f, "200 OK"),
            StatusCode::NotFound => write!(f, "404 Not Found"),
        }
    }
}
#[derive(Debug)]
struct ResponseBody {
    content_type: String,
    content_length: String,
}

impl HttpResponse {
    pub fn build_response(http_request: &HttpRequest) -> Option<HttpResponse> {
        if let Some(to_echo_back) = detect_echo(&http_request.ressource_path) {
            return Some(HttpResponse {
                status_code: StatusCode::Ok,
                content_type: "text/plain".to_string(),
                content_length: to_echo_back.len(),
                protocol_version: http_request.protocol_version,
                headers: Vec::new(),
                body: to_echo_back.to_string(),
            });
        } else {
            return None;
        }
    }

    pub fn craft_response(http_response: &HttpResponse) -> String {
        // HTTP/1.1 200 OK\r\n
        let status_line = format!(
            "{} {}",
            http_response.protocol_version, http_response.status_code
        );
        let content = format!(
            "Content-type: {}\r\nContent-Length: {}",
            http_response.content_type, http_response.content_length
        );

        format!(
            "{}\r\n{}\r\n\r\n{}",
            status_line, content, http_response.body
        )
    }
}

// GET request to the /echo/{str} endpoint
pub fn detect_echo(ressource_path: &str) -> Option<&str> {
    let parts: Vec<&str> = ressource_path.split("/").collect();

    if parts.len() != 3 {
        // Invalid request.
        // Expected /echo/{some_str} --> ["", "echo", "abc"]
        println!("Detect echo - did not found 3 parts.");
        println!("Parts: {:?}", parts);
        println!("From ressource path: {}", ressource_path);
        return None;
    } else {
        // Valid request : /echo/{some_str}  --> ["", "echo", "abc"]
        let to_echo_back = parts[2];
        dbg!(to_echo_back);
        return Some(to_echo_back);
    }
}
