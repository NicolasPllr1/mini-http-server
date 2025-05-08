use crate::encoding::ContentEncoding;
use crate::http_request::HttpMethod;
use crate::http_request::HttpRequest;
use crate::http_response::StatusCode;
use crate::http_response::{Buildable, Builder, HttpResponse};

use std::error::Error;
use std::fs;
use std::path::Path;
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

// TODO: explore/compare the trait approach for the endpoints implementations
impl Endpoints {
    /// Handles a HTTP request. Each endpoint handles requests in its own way.
    /// # Errors
    /// Endpoints can return errors.
    pub fn handle_request(
        &self,
        http_request: &HttpRequest,
        data_dir: &Path,
    ) -> Result<HttpResponse, Box<dyn Error>> {
        let mut builder = HttpResponse::builder();

        builder.with_protocol_version(http_request.protocol_version);

        let content_encoding = http_request
            .headers
            .get("Accept-Encoding")
            .and_then(|hdr_val| ContentEncoding::from_header(hdr_val));

        builder.with_content_encoding(content_encoding);

        builder.with_conn_close(!http_request.keep_alive());

        match self {
            Endpoints::UrlPath => {}

            Endpoints::Echo => {
                let to_echo_back = http_request.request_target[6..].to_string(); // '/echo/{str}'
                builder.with_content_length(to_echo_back.len());
                builder.with_body(&to_echo_back);
            }
            Endpoints::UserAgent => {
                let user_agent_body = http_request
                    .headers
                    .get("User-Agent")
                    .ok_or("User-Agent endpoint expects 'User-Agent' header")?;

                builder.with_content_length(user_agent_body.len());
                builder.with_body(user_agent_body);
            }
            Endpoints::Sleep => {
                thread::sleep(Duration::from_secs(10));
                let sleep_msg = "Good sleep!".to_string();
                builder.with_content_length(sleep_msg.len());
                builder.with_body(&sleep_msg);
            }
            Endpoints::File => match http_request.http_method {
                HttpMethod::Get => match Self::get_file_content(http_request, data_dir) {
                    Ok(file_content) => {
                        builder.with_content_type("application/octet-stream");
                        builder.with_content_length(file_content.as_bytes().len());
                        builder.with_body(&file_content);
                    }
                    Err(_) => {
                        builder.with_status_code(StatusCode::NotFound);
                    }
                },
                HttpMethod::Post => {
                    let filename = Self::get_target_filename(http_request)?;
                    let file_path = data_dir.join(filename);
                    let content = http_request
                        .body
                        .clone()
                        .ok_or("Body should have been provided")?;
                    match fs::write(file_path, content) {
                        Ok(()) => {
                            builder.with_status_code(StatusCode::Created);
                            builder.with_content_type("application/octet-stream");
                        }

                        Err(_) => {
                            builder.with_status_code(StatusCode::NotFound);
                        }
                    }
                }
            },
            Endpoints::NotAvailable => {
                builder.with_status_code(StatusCode::NotFound);
            }
        };
        Ok(builder.build())
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
            _ => Ok(Self::NotAvailable), // could Err here and map to NotAvailable higher-up ?
        }
    }
}

// Private utils
impl Endpoints {
    fn get_target_filename(http_request: &HttpRequest) -> Result<&str, Box<dyn Error>> {
        http_request
            .request_target
            .split('/')
            .last()
            .ok_or("Expected : /files/{filename}".into())
    }

    fn get_file_content(
        http_request: &HttpRequest,
        data_dir: &Path,
    ) -> Result<String, Box<dyn Error>> {
        let filename = Self::get_target_filename(http_request)?;
        let file_path = data_dir.join(filename);
        dbg!(&file_path);
        let file_content = fs::read_to_string(file_path)?;
        Ok(file_content)
    }
}
