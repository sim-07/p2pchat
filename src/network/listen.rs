use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use crate::state::state_packets::Packet;

pub async fn listen(stream: &mut TcpStream) -> Option<Packet> {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer).await {
        Ok(0) => {
            println!("Connection closed");
            None
        }
        Ok(n) => {
            let received_data = String::from_utf8_lossy(&buffer[..n]);
            match serde_json::from_str::<Packet>(&received_data) {
                Ok(packet) => Some(packet),
                Err(e) => {
                    eprintln!("Errore deserializzando pacchetto: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Stream error: {}", e);
            None
        }
    }
}
