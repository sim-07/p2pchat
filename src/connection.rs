use std::collections::HashSet;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat};
use crate::manage_packets::Packet;
use crate::send;

// pub struct Connected {
//     ip: String,
//     port: u16
// }

// impl Connected {
//     pub fn new(ip: String, port: u16) -> Self {
//         Self {
//             ip,
//             port
//         }
//     }

//     pub fn addConnection(&mut self, ip: String, port: u16) {
//         self.ip = ip;
//         self.port = port;
//     }
// }

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

    let mut buffer = [0; 4096]; // Dati ricevuti dal client
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

                    
                    let chat_clone_conn = chat.clone();
                    let chat_rec_clone = chat_received.clone();
                    tokio::spawn(async move {
                        let chat_lock_conn = chat_clone_conn.lock().await;
                        connect_to_members_received(&chat_lock_conn, &chat_rec_clone).await;
                    });

                    chat_lock.set_members(chat_received.members.clone());
                    chat_lock.print_all_messages();

                    println!("Received messages");
                }
                Packet::InitSyncRequest(new_member) => {
                    println!("{} has connected ({})", new_member.username, peer_address);

                    let arc_new_member = Arc::new(Mutex::new(new_member));
                    manage_chat::add_member(Arc::clone(&chat), arc_new_member).await;

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

async fn connect_to_members_received(chat_lock: &tokio::sync::MutexGuard<'_, Chat>, chat_received: &Chat) {
    let loc_members: HashSet<_> = chat_lock.members.iter().map(|m| &m.id).collect();

    let diff: Vec<_> = chat_received
        .members
        .iter()
        .filter(|r| !loc_members.contains(&r.id))
        .collect();

    for d in &diff {
        if let Err(e) = connect_to(&d.ip, d.port).await {
            println!("Error connecting with all members: {}", e);
        }
    }
}
