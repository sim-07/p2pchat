use std::io::{Read, Write};
use std::net::{TcpStream};
use serde::{Serialize, Deserialize};

use crate::manage_chat::{self, Chat};


pub fn listen(mut stream: TcpStream, chat: &mut Chat) {
    let mut buffer: [u8; 1024] = [0; 1024]; // Max amout of bytes from the client
    let n = stream.read(&mut buffer).expect("Connection error");

    let peer_address = stream.peer_addr().expect("Could not get peer address");
    let request = String::from_utf8_lossy(&buffer[..n]); // Convert data from buffer into utf8 string (only data actually sent, non all the 1024 bytes)

    println!("{} has entered the chat", peer_address);
    println!("Request: {}", request);

    let all_messages = manage_chat::Chat::get_all_local_messages(chat);

    let response_json = serde_json::to_string(&all_messages).expect("Failed to serialize");
    stream.write_all(response_json.as_bytes()).expect("Failed to write response");
}