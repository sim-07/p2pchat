use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat};
use crate::manage_packets::Packet;
use crate::send;

pub async fn connect_to(ip: &str, port: u16) -> tokio::io::Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address).await {
        Ok(stream) => Ok(stream),
        Err(e) => {
            println!("Failed to connect to {}: {}", address, e);
            Err(e)
        }
    }
}

pub async fn receive_packet(
    stream: &mut TcpStream,
    chat: &Arc<Mutex<Chat>>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // il tipo di ritorno dice che è thread safe

    let mut buffer = [0; 4096]; // Max amout of bytes from the client
    let peer_address = stream.peer_addr().expect("Could not get peer address");

    match stream.read(&mut buffer).await {
        Ok(0) => {
            return Err("Connection closed by peer".into());
        }
        Ok(n) => {
            let received_data = String::from_utf8_lossy(&buffer[..n]);

            let packet: Packet = serde_json::from_str::<Packet>(&received_data)?;

            match packet {
                Packet::UserMessage(message) => {
                    let mut chat_lock = chat.lock().await;

                    chat_lock.print_message(&message);
                    chat_lock.add_message(message);
                }
                Packet::Sync(chat_received) => {
                    let mut chat_lock = chat.lock().await;

                    chat_lock.set_all_messages(chat_received.all_messages.clone());
                    chat_lock.set_members(chat_received.members.clone());
                    chat_lock.print_all_messages();

                    println!("Received messages");
                }
                Packet::InitSyncRequest(new_member) => {
                    println!("{} has connected ({})", new_member.username, peer_address);

                    let arc_new_member = Arc::new(Mutex::new(new_member));
                    manage_chat::add_member(Arc::clone(&chat), arc_new_member)
                        .await;

                    let chat_lock = chat.lock().await;

                    let packet = Packet::Sync(chat_lock.clone());
                    if let Err(e) = send::send(stream, &packet).await {
                        println!("Connection error sending message: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    Ok(())
}
