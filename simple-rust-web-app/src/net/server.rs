use std::fs;
use std::time::Duration;
use async_std::prelude::*;
use std::marker::Unpin;
use async_std::io::{Read, Write};
use async_std::net::{TcpListener};
use async_std::task;
use futures::stream::StreamExt;
use super::router::{Router, Req, Res};

pub struct Server<'a> {
    listener: TcpListener,
    router: Router<'a>,
}

impl<'a> Server<'a> {
    pub async fn new(host: &str, port: &str) -> Server<'a> {
        let hostname = [&host[..], ":", &port[..]].concat();
        Server {
            listener: TcpListener::bind(hostname).await.unwrap(),
            router: Router::new(),
        }
    }

    pub async fn listen(&self) {
        self.listener
                .incoming()
                .for_each_concurrent(None, |tcpstream| async move {
                    let tcpstream = tcpstream.unwrap();
                    self.use_connection_router(tcpstream).await;
                })
                .await;
    }

    async fn use_connection_router(&self, mut stream: impl Read + Write + Unpin) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).await.unwrap();
        let req = Req::new(&buffer);
        
        let (mut status_line, filename) = match req.path {
            "/" => ("HTTP/1.1 200 OK\r\n\r\n", "hello.html"),
            "/sleep" => {
                task::sleep(Duration::from_secs(5)).await;
                ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
            },
            _ => {
                match req.path.split(|incoming_char| incoming_char == '/').last() {
                    Some(file) => ("HTTP/1.1 200 OK\r\n\r\n", file),
                    None => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
                }
            }
        };

        let contents = match fs::read_to_string(filename) {
            Ok(file_contents) => file_contents,
            Err(_) => {
                status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                fs::read_to_string("404.html").unwrap()
            }
        };

        let response = format!("{status_line}{contents}");
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    }
}

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{status_line}{contents}");
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

use super::*;
use futures::io::Error;
use futures::task::{Context, Poll};

use std::cmp::min;
use std::pin::Pin;

struct MockTcpStream {
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl Read for MockTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let size: usize = min(self.read_data.len(), buf.len());
        buf[..size].copy_from_slice(&self.read_data[..size]);
        Poll::Ready(Ok(size))
    }
}

impl Write for MockTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.write_data = Vec::from(buf);

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Unpin for MockTcpStream {}

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
