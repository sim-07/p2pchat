use tokio::net::TcpStream;
use tokio::io::Result;

pub async fn connect_to(ip: &str, port: u16, chat_id: &str) -> Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address).await {
        Ok(stream) => {
            println!("Successfully connected to {}", address);
            Ok(stream)
        }
        Err(e) => {
            println!("Failed to connect to {}: {}", address, e);
            Err(e)
        }
    }
}

pub fn request_messages(chat_id: &str, stream: &mut TcpStream) -> Result<()> {
    // inviare richiesta avente tipo system per esempio per differenziare dai messaggi

    Ok(())
}