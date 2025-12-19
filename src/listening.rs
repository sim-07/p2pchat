use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};


pub fn listen(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024]; // Max amout of bytes from the client
    stream.read(&mut buffer).expect("Connection error");

    let request = String::from_utf8_lossy(&buffer[..]); // Convert data from buffer into utf8 string

    println!("Received request: {}", request);

    let response = "Connected".as_bytes();
    stream.write(response).expect("Failed to write response");
}
