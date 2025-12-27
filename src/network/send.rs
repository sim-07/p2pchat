use tokio::io::AsyncWriteExt;

use crate::state::state_packets::Packet;

pub async fn send(stream: &mut tokio::net::tcp::OwnedWriteHalf, packet: &Packet) -> tokio::io::Result<()> {
    let mut data = serde_json::to_vec(packet).expect("Failed to serialize");

    data.push(b'\n'); // framing
    
    stream.write_all(&data).await?;
    stream.flush().await?;

    Ok(())
}

