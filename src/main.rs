use std::io::Write;
use std::io::{BufRead, BufReader};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

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
                    .collect::<Vec<&String>>();

                let response = match request_line {
                    "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
                    "GET /user-agent HTTP/1.1" => {
                        let user_agent = request_headers
                            .iter()
                            .find(|&&line| line.starts_with("User-Agent"))
                            .map(|line| line.as_str())
                            .unwrap_or("User-Agent: Unknown")
                            .split(": ")
                            .nth(1)
                            .unwrap_or_else(|| "Unknown");

                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            user_agent.len(),
                            user_agent,
                        )
                    }
                    _ if request_line.starts_with("GET /echo") => {
                        println!("echo request_line: {}", request_line);
                        let path = request_line.split(" ").nth(1).unwrap();

                        println!("path: {}", path);

                        let param = path.split("/").last().unwrap();
                        let param_len = param.len();

                        println!("param_len: {}", param_len);
                        println!("param: {}", param);

                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            param_len,
                            param,
                        )
                    }
                    _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                };

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
