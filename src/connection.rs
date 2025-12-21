use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};

pub async fn connect_to(ip: &str, port: u16) -> Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address).await {
        Ok(stream) => {
            Ok(stream)
        }
        Err(e) => {
            println!("Failed to connect to {}: {}", address, e);
            Err(e)
        }
    }
}

pub async fn receive_all_messages(stream: &mut TcpStream) -> tokio::io::Result<String> {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer).await {
        Ok(n) => {
            let json = String::from_utf8_lossy(&buffer[..n]);
            Ok(json.into_owned()) // .into_owned() converte un riferimento (&str) a un tipo posseduto (String)
        }
        Err(e) => {
            eprintln!("Error receiving chat: {}", e);
            Err(e)
        },
    }
}