use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat};
use crate::manage_packets::Packet;

pub async fn listen(mut stream: TcpStream, chat: Arc<Mutex<Chat>>) {
    let peer_address = stream.peer_addr().expect("Could not get peer address");
    println!("{} has entered the chat", peer_address);

    let mut buffer = [0; 4096]; // Max amout of bytes from the client

    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                // 0 bytes = connessione chiusa
                println!("{} disconnected", peer_address);
            }
            Ok(n) => {
                // quanto un peer si connette e manda qualcosa gli mando lo struct chat
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                let packet: Packet = serde_json::from_str::<Packet>(&received_data).unwrap();

                match packet {
                    Packet::UserMessage(message) => {
                        let mut chat_lock = chat.lock().await;

                        chat_lock.add_message(message);
                    }
                    Packet::InitRequest(new_member) => {
                        println!("{} has connected ({})", new_member.username, peer_address);

                        manage_chat::add_member(
                            Arc::clone(&chat),
                            Arc::new(Mutex::new(new_member)),
                        )
                        .await;

                        let chat_lock = chat.lock().await;
                        let response_json =
                            serde_json::to_string(&*chat_lock).expect("Failed to serialize"); // *chat_lock per ottenere lo struct dal mutex, & per non prendere ownership

                        if let Err(e) = stream.write_all(response_json.as_bytes()).await {
                            eprintln!("Failed to write response: {}", e);
                        }
                    }
                    Packet::Sync(chat_received) => {
                        let mut chat_lock = chat.lock().await;

                        chat_lock.set_all_messages(chat_received.all_messages.clone());
                        chat_lock.set_members(chat_received.members.clone());

                    },
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                return;
            }
        };
    }
}
