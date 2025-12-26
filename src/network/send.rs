use tokio::io::AsyncWriteExt;

use crate::state::state_packets::Packet;

pub async fn send(stream: &mut tokio::net::tcp::OwnedWriteHalf, packet: &Packet) -> tokio::io::Result<()> {
    let data = serde_json::to_vec(packet).expect("Failed to serialize");
    
    stream.write_all(&data).await?;
    stream.flush().await?;

    Ok(())
}

