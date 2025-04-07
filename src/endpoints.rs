use crate::encoding::ContentEncoding;
use crate::http_request::HttpMethod;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::http_response::StatusCode;

use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration; // for the 'Sleep' endpoint (used to test multi-threading)

//TODO:
// 1. use combinator to reduce explicit matching
// 2. custom error with ?
// Potential refacto with a Router (dynamic vs static dispatch)

#[derive(Debug)]
pub enum Endpoints {
    UrlPath,
    Echo,
    UserAgent,
    Sleep,
    File,
    NotAvailable,
}

impl Endpoints {
    pub fn handle_request(
        &self,
        http_request: &HttpRequest,
        data_dir: &str,
    ) -> Result<HttpResponse, Box<dyn Error>> {
        let content_encoding = http_request
            .headers
            .get("Accept-Encoding")
            .and_then(|hdr_val| ContentEncoding::from_header(hdr_val));

        match self {
            Endpoints::UrlPath => Ok(HttpResponse {
                protocol_version: http_request.protocol_version,
                status_code: StatusCode::Ok,
                content_type: "text/plain".to_string(),
                content_encoding,
                content_length: 0,
                body: None,
            }),
            Endpoints::Echo => {
                let to_echo_back = http_request.request_target[6..].to_string(); // '/echo/{str}'
                Ok(HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: to_echo_back.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(to_echo_back),
                })
            }
            Endpoints::UserAgent => {
                let user_agent_body = http_request
                    .headers
                    .get("User-Agent")
                    .expect("User-Agent endpoint expects 'User-Agent' header");
                Ok(HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: user_agent_body.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(user_agent_body.clone()),
                })
            }
            Endpoints::Sleep => {
                thread::sleep(Duration::from_secs(10));
                let msg = "Good sleep!".to_string();
                Ok(HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: msg.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(msg),
                })
            }
            Endpoints::File => match http_request.http_method {
                HttpMethod::GET => match Self::get_file_content(http_request, data_dir) {
                    Some(file_content) => Ok(HttpResponse {
                        status_code: StatusCode::Ok,
                        content_type: "application/octet-stream".to_string(),
                        content_encoding,
                        content_length: file_content.as_bytes().len(),
                        protocol_version: http_request.protocol_version,
                        body: Some(file_content),
                    }),
                    None => Ok(HttpResponse {
                        status_code: StatusCode::NotFound,
                        content_type: "text/plain".to_string(),
                        content_encoding,
                        content_length: 0,
                        protocol_version: http_request.protocol_version,
                        body: None,
                    }),
                },
                HttpMethod::POST => {
                    let filename = Self::get_target_filename(http_request).unwrap();
                    let path = format!("{}/{}", data_dir, filename);
                    let content = http_request
                        .body
                        .clone()
                        .expect("Body should have been provided");
                    match fs::write(path, content) {
                        Ok(_) => Ok(HttpResponse {
                            status_code: StatusCode::Created,
                            content_type: "application/octet-stream".to_string(),
                            content_encoding,
                            content_length: 0,
                            protocol_version: http_request.protocol_version,
                            body: None,
                        }),
                        Err(_) => Ok(HttpResponse {
                            status_code: StatusCode::NotFound,
                            content_type: "text/plain".to_string(),
                            content_encoding,
                            content_length: 0,
                            protocol_version: http_request.protocol_version,
                            body: None,
                        }),
                    }
                }
            },
            Endpoints::NotAvailable => Ok(HttpResponse {
                status_code: StatusCode::NotFound,
                content_type: "text/plain".to_string(),
                content_encoding,
                content_length: 0,
                protocol_version: http_request.protocol_version,
                body: None,
            }),
        }
    }
}

impl std::str::FromStr for Endpoints {
    type Err = String;

    fn from_str(request_target: &str) -> Result<Self, Self::Err> {
        match request_target {
            "/" => Ok(Self::UrlPath),
            s if s.starts_with("/echo/") => Ok(Self::Echo),
            "/user-agent" => Ok(Self::UserAgent),
            "/sleep" => Ok(Self::Sleep),
            s if s.starts_with("/files/") => Ok(Self::File),
            _ => Ok(Self::NotAvailable),
            // _ => Err(format!(
            //     "Request target does not match any available endpoint: {}",
            //     request_target
            // )
            // .into()),
        }
    }
}

// Private utils
impl Endpoints {
    fn get_target_filename(http_request: &HttpRequest) -> Option<&str> {
        let filename = http_request
            .request_target
            .split("/")
            .last()
            .expect("Expected : /files/{filename}");
        Some(filename)
    }

    fn get_file_content(http_request: &HttpRequest, data_dir: &str) -> Option<String> {
        let filename = Self::get_target_filename(http_request).unwrap();
        let file_path = format!("{}/{}", data_dir, filename);
        dbg!(&file_path);
        fs::read_to_string(file_path).ok()
    }
}
