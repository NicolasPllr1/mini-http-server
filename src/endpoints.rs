use crate::encoding::ContentEncoding;
use crate::http_request::HttpMethod;
use crate::http_request::HttpRequest;
use crate::http_response::ContentType;
use crate::http_response::StatusCode;
use crate::http_response::{Buildable, Builder, HttpResponse};

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration; // for the 'Sleep' endpoint (used to test multi-threading)

//TODO:
// 1. use combinator to reduce explicit matching
// 2. custom error with ?
// Potential refacto with a Router (dynamic vs static dispatch)

#[derive(Debug)]
pub enum Endpoints {
    Echo,
    UserAgent,
    Sleep,
    File,
}

#[derive(Debug)]
pub enum EndpointError {
    EndpointNotRecognized(String),
    UserAgentNotFound,
    PostBodyNotFound,
    TargetFileExtansion(String),
    ContentType(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for EndpointError {
    fn from(e: std::io::Error) -> EndpointError {
        EndpointError::Io(e)
    }
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndpointError::EndpointNotRecognized(e) => {
                write!(f, "request-target not recognized: {e}")
            }
            EndpointError::UserAgentNotFound => {
                write!(f, "user-agent header not found")
            }
            EndpointError::PostBodyNotFound => write!(f, "body not found"),

            EndpointError::ContentType(t) => {
                write!(f, "problem parsing the content-type : {t}")
            }
            EndpointError::TargetFileExtansion(filename) => {
                write!(
                    f,
                    "problem getting the target file extansion, filename is : {filename} "
                )
            }
            EndpointError::Io(e) => write!(f, "I/O on the requested file : {e}"),
        }
    }
}

impl std::error::Error for EndpointError {}

// TODO: explore/compare the trait approach for the endpoints implementations
impl Endpoints {
    /// Handles a HTTP request. Each endpoint handles requests in its own way.
    /// # Errors
    /// Endpoints can return errors.
    pub fn handle_request(
        &self,
        http_request: &HttpRequest,
        data_dir: &Path,
    ) -> Result<HttpResponse, EndpointError> {
        let mut builder = HttpResponse::builder();

        builder.with_protocol_version(http_request.protocol_version);

        let content_encoding = http_request
            .headers
            .get("Accept-Encoding")
            .and_then(|hdr_val| ContentEncoding::from_header(hdr_val));

        builder.with_content_encoding(content_encoding);

        builder.with_conn_close(!http_request.keep_alive());

        match self {
            Endpoints::Echo => {
                let to_echo_back = http_request.request_target[6..].as_bytes(); // '/echo/{str}'
                builder.with_content_length(to_echo_back.len());
                builder.with_body(to_echo_back);
            }
            Endpoints::UserAgent => {
                let user_agent_body = http_request
                    .headers
                    .get("User-Agent")
                    .ok_or(EndpointError::UserAgentNotFound)?;
                // .ok_or("User-Agent endpoint expects 'User-Agent' header")?;

                builder.with_content_length(user_agent_body.len());
                builder.with_body(user_agent_body.as_bytes());
            }
            Endpoints::Sleep => {
                thread::sleep(Duration::from_secs(10));
                let sleep_msg = "Good sleep!".as_bytes();
                builder.with_content_length(sleep_msg.len());
                builder.with_body(sleep_msg);
            }
            Endpoints::File => match http_request.http_method {
                HttpMethod::Get => match Self::get_file_content(http_request, data_dir) {
                    Ok(file_content) => {
                        let content_type = Self::get_file_content_type(http_request, data_dir)?;
                        builder.with_content_type(content_type);
                        builder.with_content_length(file_content.len());
                        builder.with_body(&file_content);
                    }
                    Err(e) => {
                        // TODO: could be more precise here, depending on the EndpointError
                        eprintln!("error getting file content: {e})");
                        builder.with_status_code(StatusCode::NotFound);
                    }
                },
                HttpMethod::Post => {
                    let filename = Self::get_target_filename(http_request);
                    let file_path = data_dir.join(filename);
                    let content = http_request
                        .body
                        .clone()
                        .ok_or(EndpointError::PostBodyNotFound)?;
                    // .ok_or("Body should have been provided")?;

                    match fs::write(file_path, content) {
                        Ok(()) => {
                            builder.with_status_code(StatusCode::Created);
                            builder.with_content_type(ContentType::OctetStream);
                        }

                        Err(_) => {
                            builder.with_status_code(StatusCode::NotFound);
                        }
                    }
                }
            },
        };
        Ok(builder.build())
    }
}

impl std::str::FromStr for Endpoints {
    type Err = EndpointError;

    fn from_str(request_target: &str) -> Result<Self, Self::Err> {
        match request_target {
            s if s.starts_with("/echo/") => Ok(Self::Echo),
            "/user-agent" => Ok(Self::UserAgent),
            "/sleep" => Ok(Self::Sleep),
            s if s.starts_with("/files/") => Ok(Self::File),
            s if s.starts_with('/') => Ok(Self::File),
            _ => {
                eprintln!("Error parsing the endpoint: {request_target}");
                Err(EndpointError::EndpointNotRecognized(request_target.into()))
            }
        }
    }
}

// Private utils
impl Endpoints {
    const DEFAULT_TARGET: &str = "index.html";

    fn get_target_filename(http_request: &HttpRequest) -> &str {
        let path = http_request.request_target.trim_start_matches('/');

        if path.is_empty() {
            return Self::DEFAULT_TARGET;
        }

        path
    }

    fn get_file_content(
        http_request: &HttpRequest,
        data_dir: &Path,
    ) -> Result<Vec<u8>, EndpointError> {
        let request_target = http_request.request_target.trim_start_matches('/');

        // Empty target: return `DEFAULT_TARGET` content if possible, else a directory listing
        if request_target.is_empty() {
            let file_path = data_dir.join(Self::DEFAULT_TARGET);
            dbg!(&file_path);

            match fs::read(file_path) {
                Ok(bytes) => Ok(bytes),
                // `ref e` to borrow the error, making the read-only need explicit. Why not, but
                // here also
                // compiles without the `ref`
                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                    dbg!("enter render dir listing");
                    Ok(Self::render_directory_listing(data_dir)?)
                }
                Err(e) => Err(EndpointError::Io(e)),
            }
        }
        // Regular code path: try to read target
        else {
            let file_path = data_dir.join(request_target);
            dbg!(&file_path);
            let file_content = fs::read(file_path)?;
            Ok(file_content)
        }
    }

    pub fn get_file_content_type(
        http_request: &HttpRequest,
        data_dir: &Path,
    ) -> Result<ContentType, EndpointError> {
        let filename = Self::get_target_filename(http_request);
        let file_path = data_dir.join(filename);
        dbg!(file_path.clone());

        let ext_str = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(EndpointError::TargetFileExtansion(filename.into()))?;

        let content_type = ext_str
            .parse::<ContentType>()
            .map_err(|()| EndpointError::ContentType(ext_str.into()))?;

        dbg!(ext_str);
        dbg!(content_type);

        Ok(content_type)
    }

    /// o3 generated: tiny, dependency‑free directory‑listing generator.
    fn render_directory_listing(dir: &Path) -> std::io::Result<Vec<u8>> {
        let mut html = Vec::new();

        writeln!(
            html,
            "<!doctype html><meta charset=\"utf-8\">\
         <title>Index of {}</title><h1>Index of {}</h1><ul>",
            dir.display(),
            dir.display()
        )?;

        dbg!("list dir: trying to read {dir}");
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            } // hide dotfiles

            let display = if entry.file_type()?.is_dir() {
                format!("{name}/")
            } else {
                name.to_string()
            };

            writeln!(html, r#"<li><a href="{display}">{display}</a></li>"#)?;
        }

        writeln!(html, "</ul>")?;
        Ok(html)
    }
}
