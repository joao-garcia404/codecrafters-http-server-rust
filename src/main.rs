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
                let request_line = buf_reader.lines().next().unwrap().unwrap();

                let response = match request_line.as_str() {
                    "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
                    _ if request_line.starts_with("GET /echo/") => {
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
