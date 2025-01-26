use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, fs};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                let _handle = std::thread::spawn(move || {
                    let env_args = env::args().collect::<Vec<String>>();
                    let request = parse_request(&stream);

                    let response = match request.line.as_str() {
                        "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
                        "GET /user-agent HTTP/1.1" => {
                            let user_agent = request.get_header("User-Agent").unwrap_or("Unknown");

                            let mut response = Response::new(ResponseStatus::Ok);

                            response.add_header("Content-Type", "text/plain");
                            response.add_header("Content-Length", &user_agent.len().to_string());
                            response.add_body(user_agent);

                            response.to_string()
                        }
                        _ if request.line.as_str().starts_with("GET /echo") => {
                            let path = request.get_path().unwrap();
                            let param = path.split("/").last().unwrap();
                            let param_len = param.len();

                            let mut response = Response::new(ResponseStatus::Ok);

                            response.add_header("Content-Type", "text/plain");
                            response.add_header("Content-Length", &param_len.to_string());

                            let compression_schemes = request.get_header("Accept-Encoding");

                            if let Some(compression_schemes) = compression_schemes {
                                response.compress_headers(compression_schemes);
                            }

                            response.add_body(param);
                            response.to_string()
                        }
                        _ if request.line.as_str().starts_with("GET /files") => {
                            let path = request.get_path().unwrap();
                            let file_name = path.split("/").last().unwrap();

                            let file_directory = match env_args.get(2) {
                                Some(file_directory) => file_directory.to_string(),
                                None => "/tmp".to_string(),
                            };

                            let file_pathname = format!("/{}/{}", &file_directory, file_name);
                            let file = std::fs::read_to_string(file_pathname);

                            match file {
                                Ok(file) => {
                                    let file_len = file.len();

                                    let mut response = Response::new(ResponseStatus::Ok);

                                    response.add_header("Content-Type", "application/octet-stream");
                                    response.add_header("Content-Length", &file_len.to_string());

                                    response.add_body(&file);

                                    response.to_string()
                                }
                                Err(_) => Response::new(ResponseStatus::NotFound).to_string(),
                            }
                        }
                        _ if request.line.as_str().starts_with("POST /files") => {
                            let path = request.get_path().unwrap();
                            let file_name = path.split("/").last().unwrap();

                            let file_directory = match env_args.get(2) {
                                Some(file_directory) => file_directory.to_string(),
                                None => "/tmp".to_string(),
                            };

                            if let Some(body) = &request.body {
                                let body = body.to_string();
                                let file_pathname = format!("{}{}", &file_directory, file_name);

                                let write_result = fs::write(file_pathname, body);

                                if write_result.is_err() {
                                    return;
                                }

                                Response::new(ResponseStatus::Created).to_string()
                            } else {
                                Response::new(ResponseStatus::NotFound).to_string()
                            }
                        }
                        _ => Response::new(ResponseStatus::NotFound).to_string(),
                    };

                    stream.write_all(response.as_bytes()).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

enum ResponseStatus {
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

struct Response {
    status: ResponseStatus,
    headers: Vec<String>,
    body: Option<String>,
}

impl ToString for Response {
    fn to_string(&self) -> String {
        let status = self.status.to_string();
        let headers = self.headers.join("\r\n");

        if let Some(body) = &self.body {
            return format!("{}\r\n{}\r\n\r\n{}", status, headers, body);
        }

        format!("{}\r\n{}\r\n\r\n", status, headers)
    }
}

impl Response {
    fn new(status: ResponseStatus) -> Response {
        Response {
            status,
            headers: Vec::new(),
            body: None,
        }
    }

    fn add_header(&mut self, key: &str, value: &str) {
        self.headers.push(format!("{}: {}", key, value));
    }

    fn compress_headers(&mut self, schemes: &str) {
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

    fn add_body(&mut self, body: &str) {
        self.body = Some(body.to_string());
    }
}

struct Request {
    line: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl Request {
    fn get_path(&self) -> Option<&str> {
        self.line.split(" ").nth(1)
    }

    fn get_header(&self, key: &str) -> Option<&str> {
        let raw = self.headers.iter().find(|&item| item.starts_with(key));

        match raw {
            Some(header) => {
                let header = header.split(": ").nth(1);
                header
            }
            None => None,
        }
    }
}

fn parse_request(mut stream: &TcpStream) -> Request {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request = Vec::new();

    for line in buf_reader.by_ref().lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }

        request.push(line);
    }

    let request_line = request.first().unwrap().as_str();
    let request_headers = request
        .iter()
        .skip(1)
        .take_while(|line| {
            let line = line.to_string();
            line != ""
        })
        .map(|line| line.to_string())
        .collect::<Vec<String>>();

    let mut content_length = 0;

    for header in &request_headers {
        if header.starts_with("Content-Length:") {
            content_length = header.split(": ").nth(1).unwrap().parse::<usize>().unwrap();
            break;
        }
    }

    let mut body = String::new();

    buf_reader
        .by_ref()
        .take(content_length as u64)
        .read_to_string(&mut body)
        .unwrap();

    let request_body = if body.is_empty() { None } else { Some(body) };

    Request {
        line: request_line.to_string(),
        headers: request_headers,
        body: request_body,
    }
}
