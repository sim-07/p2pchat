use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat};

pub async fn listen(mut stream: TcpStream, chat: Arc<Mutex<Chat>>) {
    let peer_address = stream.peer_addr().expect("Could not get peer address");
    println!("{} has entered the chat", peer_address);

    let mut buffer: [u8; 4096] = [0; 4096]; // Max amout of bytes from the client
    let n = match stream.read(&mut buffer).await {
        Ok(0) => return, // o bytes = connessione chiusa
        Ok(n) => n,
        Err(e) => {
            eprintln!("Read error: {}", e);
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..n]); // Convert data from buffer into utf8 string (only data actually sent, non all the 1024 bytes)

    println!("Request: {}", request);

    let chat_lock = chat.lock().await;
    let response_json = serde_json::to_string(&*chat_lock).expect("Failed to serialize"); // *chat_lock per ottenere lo struct dal mutex, & per non prendere ownership

    if let Err(e) = stream.write_all(response_json.as_bytes()).await {
        eprintln!("Failed to write response: {}", e);
    }
}
