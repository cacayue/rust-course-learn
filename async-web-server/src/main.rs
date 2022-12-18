use async_std;
use async_std::io::ReadExt;
use async_std::net::{TcpListener };
use async_std::task::spawn;
use async_web_server::*;
use futures::{AsyncWriteExt, StreamExt};
use std::fs;
use std::time::Duration;
use async_std::io::Read;
use async_std::io::Write;

#[async_std::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    listener
        .incoming()
        .for_each_concurrent(None, |tcp_stream| async move {
            let stream = tcp_stream.unwrap();
            spawn(handle_connection(stream));
        })
        .await;

    print!("Shutting down");
}

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let mut file_type = HtmlFile::NotFound;

    if buffer.starts_with(GET) {
        file_type = HtmlFile::OK
    } else if buffer.starts_with(SLEEP) {
        async_std::task::sleep(Duration::from_secs(5)).await;
        file_type = HtmlFile::SLEEP
    }

    build_content(file_type, stream).await;
}

async fn build_content(file: HtmlFile, mut stream: impl Read + Write + Unpin) {
    let (path, status_line) = match file {
        HtmlFile::OK => ("hello.html", "HTTP/1.1 200 OK"),
        HtmlFile::SLEEP => ("hello.html", "HTTP/1.1 200 OK"),
        _ => ("404.html", "HTTP/1.1 404 NOT FOUND"),
    };

    let contents = fs::read_to_string(path).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

#[async_std::test]
async fn test_handle_connection() {
    let input_bytes = b"GET / HTTP/1.1\r\n";
    let mut contents = vec![0u8; 1024];
    contents[..input_bytes.len()].clone_from_slice(input_bytes);
    let mut stream = MockTcpStream {
        read_data: contents,
        write_data: Vec::new(),
    };

    handle_connection(&mut stream).await;
    let mut buf = [0u8; 1024];
    stream.read(&mut buf).await.unwrap();

    let expected_contents = fs::read_to_string("hello.html").unwrap();
    let expected_response = format!("HTTP/1.1 200 OK\r\n\r\n{}", expected_contents);
    assert!(stream.write_data.starts_with(expected_response.as_bytes()));
}