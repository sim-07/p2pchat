use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use crate::manage_packets::Packet;

pub async fn send(stream: &mut TcpStream, packet: &Packet) -> tokio::io::Result<()> {
    let data = serde_json::to_vec(packet).expect("Failed to serialize");
    
    stream.write_all(&data).await?;
    stream.flush().await?;

    Ok(())
}