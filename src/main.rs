use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                let _handle = std::thread::spawn(move || {
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
                            response.add_body(param);

                            response.to_string()
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
    NotFound,
}

impl ResponseStatus {
    fn to_string(&self) -> &str {
        match self {
            ResponseStatus::Ok => "HTTP/1.1 200 OK",
            ResponseStatus::NotFound => "HTTP/1.1 404 Not Found",
        }
    }
}
struct Response {
    status: ResponseStatus,
    headers: Vec<String>,
    body: Option<String>,
}

impl Response {
    fn new(status: ResponseStatus) -> Response {
        Response {
            status: status,
            headers: Vec::new(),
            body: None,
        }
    }

    fn add_header(&mut self, key: &str, value: &str) {
        self.headers.push(format!("{}: {}", key, value));
    }

    fn add_body(&mut self, body: &str) {
        self.body = Some(body.to_string());
    }

    fn to_string(&self) -> String {
        let status = self.status.to_string();
        let headers = self.headers.join("\r\n");

        if let Some(body) = &self.body {
            return format!("{}\r\n{}\r\n\r\n{}", status, headers, body);
        }

        format!("{}\r\n{}\r\n\r\n", status, headers)
    }
}

struct Request {
    line: String,
    headers: Vec<String>,
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
    let buf_reader = BufReader::new(&mut stream);
    let mut request = Vec::new();

    for line in buf_reader.lines() {
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

    Request {
        line: request_line.to_string(),
        headers: request_headers,
    }
}
