// <protocol> <status-code> <status-text>
// e.g. HTTP/1.1 200 OK <status-text>

use crate::html_commons::HttpVersion;
use crate::html_request::HttpRequest;
use std::fmt::Display;

#[derive(Debug)]
pub struct HttpResponse {
    protocol_version: HttpVersion,
    status_code: StatusCode,
    content_type: String,
    content_length: usize,
    body: Option<String>,
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

pub enum Endpoints {
    UrlPath,
    Echo,
    UserAgent,
}

pub fn parse_request_target(request_target: &str) -> Option<Endpoints> {
    match request_target {
        "/" => Some(Endpoints::UrlPath),
        s if s.starts_with("/echo/") => Some(Endpoints::Echo),
        "/user-agent" => Some(Endpoints::UserAgent),
        _ => None,
    }
}

impl HttpResponse {
    pub fn build_response(http_request: &HttpRequest) -> HttpResponse {
        let endpoint_requested = parse_request_target(&http_request.request_target);

        match endpoint_requested {
            Some(Endpoints::UrlPath) => HttpResponse {
                status_code: StatusCode::Ok,
                content_type: "text/plain".to_string(),
                content_length: 0,
                protocol_version: http_request.protocol_version,
                body: None,
            },
            Some(Endpoints::Echo) => {
                let to_echo_back = http_request.request_target[6..].to_string(); // '/echo/{str}'
                HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_length: to_echo_back.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(to_echo_back),
                }
            }
            Some(Endpoints::UserAgent) => {
                let user_agent_body = http_request
                    .headers
                    .as_ref()
                    .expect("User-Agent endpoint requires non-empty heades")
                    .get("User-Agent")
                    .expect("User-Agent endpoint expects 'User-Agent' header");
                HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_length: user_agent_body.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(user_agent_body.clone()),
                }
            }
            None => HttpResponse {
                status_code: StatusCode::NotFound,
                content_type: "text/plain".to_string(),
                content_length: 0,
                protocol_version: http_request.protocol_version,
                body: None,
            },
        }
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content_length == 0 {
            match self.status_code {
                StatusCode::Ok => write!(f, "HTTP/1.1 200 OK\r\n\r\n"),
                StatusCode::NotFound => write!(f, "HTTP/1.1 404 Not Found\r\n\r\n"),
            }
        } else if let Some(body) = &self.body {
            write!(
                f,
                "{} {}\r\nContent-type: {}\r\nContent-Length: {}\r\n\r\n{}",
                self.protocol_version,
                self.status_code,
                self.content_type,
                self.content_length,
                body
            )
        } else {
            write!(
                f,
                "{} {}\r\nContent-type: {}\r\nContent-Length: {}\r\n\r\n",
                self.protocol_version, self.status_code, self.content_type, self.content_length,
            )
        }
    }
}
