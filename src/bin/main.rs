use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use learnserver::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("Failed to bind TCP socket on port 7878 at 127.0.0.1");
    // connection to a port to listen -> binding

    let pool = ThreadPool::new(4);

    println!("Listening for connections on port 7878");
    for stream in listener.incoming() {
        let stream = stream.expect("Error while reading to the stream");

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [1; 1024];
    stream
        .read(&mut buffer)
        .expect("Unable to read the stream into the buffer");

    println!(
        "Request: {}",
        (String::from_utf8_lossy(&buffer[..]))
            .split("\r\n")
            .next()
            .expect("No http request found")
    );
    // lossy -> U+FFFD Replacement Character

    let get_root = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get_root) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(format!("html/{}", filename))
        .unwrap_or_else(|_| panic!("File {} not found under html directory", filename));

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream
        .write_all(response.as_bytes())
        .expect("Unable to write response to the stream");
    stream.flush().expect("Not all bytes could be wrriten");
}
