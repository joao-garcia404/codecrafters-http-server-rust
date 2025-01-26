use std::collections::HashMap;
use std::io::Write;

use crate::utils::gzip::compress as gzip_compress;

pub enum ResponseStatus {
    Ok,
    Created,
    NotFound,
}

impl ToString for ResponseStatus {
    fn to_string(&self) -> String {
        match self {
            ResponseStatus::Ok => "HTTP/1.1 200 OK".to_owned(),
            ResponseStatus::Created => "HTTP/1.1 201 Created".to_owned(),
            ResponseStatus::NotFound => "HTTP/1.1 404 Not Found".to_owned(),
        }
    }
}

pub struct Response {
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: ResponseStatus) -> Response {
        Response {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn compress(&mut self, schemes: &str) {
        let schemes = schemes.split(",");

        for scheme in schemes {
            match scheme.trim() {
                "gzip" => {
                    self.add_header("Content-Encoding", "gzip");
                    return;
                }
                _ => {}
            }
        }
    }

    pub fn add_body(&mut self, body: &[u8]) {
        let compression_header = self.headers.get_key_value("Content-Encoding");

        if let Some((_, scheme)) = compression_header {
            match scheme.as_str() {
                "gzip" => {
                    let compressed = gzip_compress(body).unwrap_or_else(|_| body.to_vec());
                    self.body = compressed
                }
                _ => self.body = body.to_vec(),
            }

            return;
        }

        self.body = body.to_vec();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self.status {
            ResponseStatus::Ok => {
                write!(bytes, "HTTP/1.1 200 OK\r\n").unwrap();
            }
            ResponseStatus::Created => {
                write!(bytes, "HTTP/1.1 201 Created\r\n").unwrap();
            }
            ResponseStatus::NotFound => {
                write!(bytes, "HTTP/1.1 404 Not Found\r\n").unwrap();
            }
        }

        for (key, value) in &self.headers {
            write!(bytes, "{}: {}\r\n", key, value).unwrap();
        }

        if self.body.len() > 0 {
            write!(bytes, "Content-Length: {}\r\n", self.body.len()).unwrap();
        }

        write!(bytes, "\r\n").unwrap();

        bytes.extend_from_slice(&self.body);
        bytes
    }
}
