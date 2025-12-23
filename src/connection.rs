use std::collections::HashSet;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use crate::listen::listen;
use crate::manage_chat::{self, Chat, Message};
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
    tx: broadcast::Sender<Message>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = [0; 4096]; // Dati ricevuti dal client

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

                    let diff: Vec<manage_chat::Member> =
                        get_members_diff(&chat_lock, &chat_received);

                    println!("DIFF: {:?} ++++++++++++++++++++++++++++++++", diff);

                    for m in &diff {
                        chat_lock.add_member(m.clone());
                    }

                    if diff.len() >= 1 {
                        let chat_clone = Arc::clone(chat);
                        tokio::spawn(async move {
                            for m in diff {
                                if let Ok(stream) = connect_to(&m.ip, m.port).await { // PROBLEMA: server vede client come nuovo membro e allora si connette, creando doppia connessione
                                    let chat_clone_in = Arc::clone(&chat_clone);
                                    listen(stream, chat_clone_in, tx.clone());
                                } else {
                                    eprintln!(
                                        "Failed to connect to {} at {}:{}",
                                        m.username, m.ip, m.port
                                    );
                                }
                            }
                        });
                    }
                }
                Packet::InitSyncRequest => {
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

fn get_members_diff(
    chat_lock: &tokio::sync::MutexGuard<'_, Chat>,
    chat_received: &Chat,
) -> Vec<manage_chat::Member> {
    let loc_members: HashSet<_> = chat_lock.members.iter().map(|m| &m.id).collect();

    chat_received
        .members
        .iter()
        .filter(|r| !loc_members.contains(&r.id))
        .map(|r| (**r).clone()) // (*r) dà Arc, (**r) dà Member.
        .collect()
}
