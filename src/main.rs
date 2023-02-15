#![allow(unused)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Serialize};
use serde_json::json;

const HTTP_HOST: &str = "0.0.0.0";
const HTTP_PORT: &str = "8082";

fn handle(mut stream: TcpStream, connections: &Mutex<i32>) {
    let mut id  = connections.lock().unwrap();
    *id += 1;

    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer);
    let request = String::from_utf8_lossy(&buffer[..]);
    let mut split = request.split_whitespace();
    let method = split.next().unwrap().to_string().to_uppercase();
    let path = split.next().unwrap();
    let headers: Vec<&str> = request.lines().filter(|line| line.contains(":")).collect();
    let data = Data{
        headers,
        method,
        path
    };
    let response_json = json!(data);
    let response_str = serde_json::to_string(&response_json).unwrap();

    let mut response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", response_str.len(), response_str);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    stream.shutdown(std::net::Shutdown::Both).unwrap();
}

#[derive(Serialize)]
struct Data<'a> {
    headers: Vec<&'a str>,
    method: String,
    path: &'a str
}

fn main() {

    let address = format!("{}:{}", HTTP_HOST, HTTP_PORT);
    let listener = TcpListener::bind(address).unwrap();
    let mut connections = Arc::new(Mutex::new(0));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let con = connections.clone();
                std::thread::spawn(move || {
                    println!("Thread id: {:?}", thread::current().id());
                    handle(stream, &con);
                });

            }
            Err(e) => {
                eprintln!("failed to establish connection: {}", e);
            }
        }
    }
}
