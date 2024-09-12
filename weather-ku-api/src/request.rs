use std::collections::HashMap;

pub type Headers = HashMap<String, String>;

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

#[derive(Debug)]
pub enum RequestError {
    UnsupportedMethod(Method),
    InvalidPath(String),
    InvalidVersion(String),
    InvalidHeader(String),
}

pub struct Request {
    method: Method,
    path: String,
    version: String,
    headers: Headers,
    body: Option<String>,
}
impl Request {
    pub fn from_string(request: String) -> Result<Request, RequestError> {
        let request_lines: Vec<&str> = request.lines().collect();
        let request_line: Vec<&str> = request_lines[0].split_whitespace().collect();
        let method = request_line[0];
        let path = request_line[1];
        let version = request_line[2];
        let mut headers = HashMap::new();
        for i in 1..request_lines.len() {
            if request_lines[i].is_empty() {
                break;
            }
            let header_line = request_lines[i].split_once(": ");

            if header_line.is_none() {
                break;
            }

            let header_line = header_line.unwrap();
            headers.insert(header_line.0, header_line.1);
        }
    
        
    }
}
