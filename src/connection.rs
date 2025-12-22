use tokio::io::{AsyncReadExt, Result};
use tokio::net::TcpStream;

use crate::manage_chat::Chat;

pub async fn connect_to(ip: &str, port: u16) -> Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address).await {
        Ok(stream) => Ok(stream),
        Err(e) => {
            println!("Failed to connect to {}: {}", address, e);
            Err(e)
        }
    }
}

pub async fn receive_init_chat(stream: &mut TcpStream) -> tokio::io::Result<Chat> {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer).await {
        Ok(0) => Err(tokio::io::Error::new(
            tokio::io::ErrorKind::ConnectionAborted,
            "Peer disconnected",
        )),
        Ok(n) => {
            let received_data = String::from_utf8_lossy(&buffer[..n]);
            match serde_json::from_str::<Chat>(&received_data) {
                Ok(chat) => {
                    Ok(chat)
                }
                Err(e) => {
                    eprintln!("Member data not valid: {}", e);
                    Err(tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, e))
                }
            }
        }
        Err(e) => {
            eprintln!("Error receiving chat: {}", e);
            Err(e)
        }
    }
}
