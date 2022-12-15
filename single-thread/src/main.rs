use std::io::prelude::*;
use std::{
    fs,
    net::{TcpListener, TcpStream},
};

const GET: &[u8] = b"GET / HTTP/1.1\r\n";

enum HtmlFile {
    OK,
    NotFound
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(GET) {
        build_content(HtmlFile::OK, stream);
    } else {
        build_content(HtmlFile::NotFound, stream);
    }

    //let response = "HTTP/1.1 200 OK\r\n\r\n";
    //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

fn build_content(file: HtmlFile, mut stream: TcpStream) {
    let (path, status_line) = match file {
        HtmlFile::OK => ("hello.html", "HTTP/1.1 200 OK"),
        HtmlFile::NotFound => ("404.html","HTTP/1.1 404 NOT FOUND")
    };

    let contents = fs::read_to_string(path).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
