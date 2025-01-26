use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

pub struct Request {
    pub line: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

impl Request {
    pub fn get_path(&self) -> Option<&str> {
        self.line.split(" ").nth(1)
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        let raw = self.headers.iter().find(|&item| item.starts_with(key));

        match raw {
            Some(header) => {
                let header = header.split(": ").nth(1);
                header
            }
            None => None,
        }
    }

    pub fn parse(mut stream: &TcpStream) -> Self {
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

        Self {
            line: request_line.to_string(),
            headers: request_headers,
            body: request_body,
        }
    }
}
