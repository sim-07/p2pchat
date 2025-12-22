use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use crate::manage_packets::Packet;

pub async fn send(stream: &mut TcpStream, packet: &Packet) {
    let data = serde_json::to_vec(packet).expect("Failed to serialize");
    if let Err(e) = stream.write_all(&data).await {
        eprintln!("Failed to send data: {}", e);
        return;
    }
}