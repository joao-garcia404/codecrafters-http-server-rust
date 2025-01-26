use std::io::Write;
use std::net::TcpListener;
use std::{env, fs};

use crate::request::Request;
use crate::response::{Response, ResponseStatus};

pub mod request;
pub mod response;
pub mod utils;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                let _handle = std::thread::spawn(move || {
                    let env_args = env::args().collect::<Vec<String>>();
                    let request = Request::parse(&stream);

                    let response = match request.line.as_str() {
                        "GET / HTTP/1.1" => Response::new(ResponseStatus::Ok).to_bytes(),
                        "GET /user-agent HTTP/1.1" => {
                            let user_agent = request
                                .get_header("User-Agent")
                                .unwrap_or("Unknown")
                                .as_bytes();

                            let mut response = Response::new(ResponseStatus::Ok);

                            response.add_header("Content-Type", "text/plain");
                            // response.add_header("Content-Length", &user_agent.len().to_string());
                            response.add_body(user_agent);

                            response.to_bytes()
                        }
                        _ if request.line.as_str().starts_with("GET /echo") => {
                            let path = request.get_path().unwrap();
                            let param = path.split("/").last().unwrap().as_bytes();
                            // let param_len = param.len();

                            let mut response = Response::new(ResponseStatus::Ok);

                            response.add_header("Content-Type", "text/plain");
                            // response.add_header("Content-Length", &param_len.to_string());

                            let compression_schemes = request.get_header("Accept-Encoding");

                            if let Some(compression_schemes) = compression_schemes {
                                response.compress(compression_schemes);
                            }

                            response.add_body(param);

                            response.to_bytes()
                        }
                        _ if request.line.as_str().starts_with("GET /files") => {
                            let path = request.get_path().unwrap();
                            let file_name = path.split("/").last().unwrap();

                            let file_directory = match env_args.get(2) {
                                Some(file_directory) => file_directory.to_string(),
                                None => "/tmp".to_string(),
                            };

                            let file_pathname = format!("/{}/{}", &file_directory, file_name);
                            let file = std::fs::read(file_pathname);

                            match file {
                                Ok(file) => {
                                    // let file_len = file.len();

                                    let mut response = Response::new(ResponseStatus::Ok);

                                    response.add_header("Content-Type", "application/octet-stream");
                                    // response.add_header("Content-Length", &file_len.to_string());

                                    response.add_body(&file);

                                    response.to_bytes()
                                }
                                Err(_) => Response::new(ResponseStatus::NotFound).to_bytes(),
                            }
                        }
                        _ if request.line.as_str().starts_with("POST /files") => {
                            let path = request.get_path().unwrap();
                            let file_name = path.split("/").last().unwrap();

                            println!("file name {}", file_name);

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

                                Response::new(ResponseStatus::Created).to_bytes()
                            } else {
                                Response::new(ResponseStatus::NotFound).to_bytes()
                            }
                        }
                        _ => Response::new(ResponseStatus::NotFound).to_bytes(),
                    };

                    stream.write_all(&response).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
