use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use crate::manage_chat::{Chat, Message};
use crate::manage_packets::Packet;
use crate::send;

pub fn listen(
    mut stream: TcpStream,
    chat: Arc<Mutex<Chat>>,
    tx: broadcast::Sender<Message>,
) {
    let mut rx = tx.subscribe();

    let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "Unknown".to_string());
    println!("[DEBUG] Starting listen task for peer: {}", peer_addr);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                net = crate::connection::receive_packet(&mut stream, &chat, tx.clone()) => {
                    match net {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Connection lost: {}", e);
                            break;
                        }
                    }
                }

                loc = rx.recv() => {
                    match loc {
                        Ok(message) => {
                            println!("[DEBUG] Sending message to {}: {:?}", peer_addr, message.text);

                            let packet = Packet::UserMessage(message);

                            if let Err(e) = send::send(&mut stream, &packet).await { 
                                println!("Failed to send to peer: {}", e);
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            println!("Slow connection");
                        },
                        Err(_) => break,
                    }
                }
            }
        }
    });
}
