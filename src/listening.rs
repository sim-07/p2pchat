use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat};


pub async fn listen(mut stream: TcpStream, chat: Arc<Mutex<Chat>>) {
    let mut buffer: [u8; 1024] = [0; 1024]; // Max amout of bytes from the client
    let n = match stream.read(&mut buffer).await {
        Ok(0) => return, // o bytes = connessione chiusa
        Ok(n) => n,
        Err(e) => {
            eprintln!("Read error: {}", e);
            return;
        }
    };

    let peer_address = stream.peer_addr().expect("Could not get peer address");
    let request = String::from_utf8_lossy(&buffer[..n]); // Convert data from buffer into utf8 string (only data actually sent, non all the 1024 bytes)

    println!("{} has entered the chat", peer_address);
    println!("Request: {}", request);

    let chat_lock = chat.lock().await;
    let all_messages = chat_lock.get_all_local_messages();

    let response_json = serde_json::to_string(&all_messages).expect("Failed to serialize");
    if let Err(e) = stream.write_all(response_json.as_bytes()).await {
        eprintln!("Failed to write response: {}", e);
    }
}