use crate::encoding::ContentEncoding;
use crate::http_commons::HttpVersion;
use crate::http_request::HttpMethod;
use crate::http_request::HttpRequest;
use std::fmt::Display;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct HttpResponse {
    protocol_version: HttpVersion,
    status_code: StatusCode,
    content_type: String,
    content_length: usize,
    content_encoding: Option<ContentEncoding>,
    body: Option<String>,
}

#[derive(Debug)]
enum StatusCode {
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

pub enum Endpoints {
    UrlPath,
    Echo,
    UserAgent,
    Sleep,
    File,
}

fn parse_request_target(request_target: &str) -> Option<Endpoints> {
    match request_target {
        "/" => Some(Endpoints::UrlPath),
        s if s.starts_with("/echo/") => Some(Endpoints::Echo),
        "/user-agent" => Some(Endpoints::UserAgent),
        "/sleep" => Some(Endpoints::Sleep),
        s if s.starts_with("/files/") => Some(Endpoints::File),
        _ => None,
    }
}

fn get_target_filename(http_request: &HttpRequest) -> Option<&str> {
    let filename = http_request
        .request_target
        .split("/")
        .last()
        .expect("Expected : /files/{filename}");
    Some(filename)
}

fn get_file_content(http_request: &HttpRequest, data_dir: &str) -> Option<String> {
    let filename = get_target_filename(http_request).unwrap();
    let file_path = format!("{}/{}", data_dir, filename);
    dbg!(&file_path);
    fs::read_to_string(file_path).ok()
}

fn parse_encoding_scheme(encoding_schemes_raw_str: &str) -> Option<ContentEncoding> {
    // encoding_schemes : either a single value OR a list of values
    let encoding_schemes: Vec<&str> = encoding_schemes_raw_str
        .split(",")
        .map(|s| s.trim())
        .collect();

    if encoding_schemes.len() == 1 {
        // Got a *single* compression scheme
        match encoding_schemes[0] {
            compression_encoding if compression_encoding == "gzip" => Some(ContentEncoding::GZip),
            other_comp_scheme => {
                println!("Encoding scheme is not gzip: {}", other_comp_scheme);
                None
            }
        }
    } else {
        // Got a *list* of compression schemes
        if encoding_schemes.contains(&"gzip") {
            println!(
                "Found gzip in the list of compression schemes: {:?}",
                encoding_schemes
            );
            return Some(ContentEncoding::GZip);
        } else {
            // Only gzip is available for now
            println!(
                "Did NOT found gzip in the list of compression schemes: {:?}",
                encoding_schemes
            );
            return None;
        }
    }
}

impl HttpResponse {
    pub fn build_response(http_request: &HttpRequest, data_dir: &str) -> HttpResponse {
        let endpoint_requested = parse_request_target(&http_request.request_target);
        let content_encoding = http_request
            .headers
            .get("Accept-Encoding")
            .map_or(None, |s| parse_encoding_scheme(&s));

        match endpoint_requested {
            Some(Endpoints::UrlPath) => HttpResponse {
                status_code: StatusCode::Ok,
                content_type: "text/plain".to_string(),
                content_encoding,
                content_length: 0,
                protocol_version: http_request.protocol_version,
                body: None,
            },
            Some(Endpoints::Echo) => {
                let to_echo_back = http_request.request_target[6..].to_string(); // '/echo/{str}'
                HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: to_echo_back.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(to_echo_back),
                }
            }
            Some(Endpoints::UserAgent) => {
                let user_agent_body = http_request
                    .headers
                    .get("User-Agent")
                    .expect("User-Agent endpoint expects 'User-Agent' header");
                HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: user_agent_body.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(user_agent_body.clone()),
                }
            }
            Some(Endpoints::Sleep) => {
                thread::sleep(Duration::from_secs(10));
                let msg = "Good sleep!".to_string();
                HttpResponse {
                    status_code: StatusCode::Ok,
                    content_type: "text/plain".to_string(),
                    content_encoding,
                    content_length: msg.len(),
                    protocol_version: http_request.protocol_version,
                    body: Some(msg),
                }
            }
            Some(Endpoints::File) => match http_request.http_method {
                HttpMethod::GET => match get_file_content(&http_request, data_dir) {
                    Some(file_content) => HttpResponse {
                        status_code: StatusCode::Ok,
                        content_type: "application/octet-stream".to_string(),
                        content_encoding,
                        content_length: file_content.as_bytes().len(),
                        protocol_version: http_request.protocol_version,
                        body: Some(file_content),
                    },
                    None => HttpResponse {
                        status_code: StatusCode::NotFound,
                        content_type: "text/plain".to_string(),
                        content_encoding,
                        content_length: 0,
                        protocol_version: http_request.protocol_version,
                        body: None,
                    },
                },
                HttpMethod::POST => {
                    let filename = get_target_filename(&http_request).unwrap();
                    let path = format!("{}/{}", data_dir, filename);
                    let content = http_request
                        .body
                        .clone()
                        .expect("Body should have been provided");
                    match fs::write(path, content) {
                        Ok(_) => HttpResponse {
                            status_code: StatusCode::Created,
                            content_type: "application/octet-stream".to_string(),
                            content_encoding,
                            content_length: 0,
                            protocol_version: http_request.protocol_version,
                            body: None,
                        },
                        Err(_) => HttpResponse {
                            status_code: StatusCode::NotFound,
                            content_type: "text/plain".to_string(),
                            content_encoding,
                            content_length: 0,
                            protocol_version: http_request.protocol_version,
                            body: None,
                        },
                    }
                }
            },
            None => HttpResponse {
                status_code: StatusCode::NotFound,
                content_type: "text/plain".to_string(),
                content_encoding,
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
            write!(f, "{} {}\r\n\r\n", self.protocol_version, self.status_code)
        } else {
            match &self.content_encoding {
                Some(encoding_scheme) => {
                    write!(
                        f,
                        "{} {}\r\nContent-type: {}\r\n{}\r\nContent-Length: {}\r\n\r\n{:02x?}",
                        self.protocol_version,
                        self.status_code,
                        self.content_type,
                        encoding_scheme,
                        self.content_length,
                        encoding_scheme.encode_body(&self.body.as_deref().unwrap_or_default()),
                    )
                }
                None => {
                    write!(
                        f,
                        "{} {}\r\nContent-type: {}\r\nContent-Length: {}\r\n\r\n{}",
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
