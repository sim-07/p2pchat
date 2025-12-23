use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use crate::manage_chat::{Chat, Message};
use crate::manage_packets::Packet;
use crate::{connection, send};

pub async fn listen(
    mut stream: TcpStream,
    chat: Arc<Mutex<Chat>>,
    mut rx: broadcast::Receiver<Message>,
) {
    let peer_address = stream.peer_addr().expect("Could not get peer address");
    println!("{} has entered the chat", peer_address);

    loop {
        tokio::select! {
            local_mes = rx.recv() => {
                match local_mes {
                    Ok(message) => {
                        let packet = Packet::UserMessage(message);

                        if let Err(e) = send::send(&mut stream, &packet).await {
                            println!("Connection error sending message: {}", e);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        println!("Slow connection");
                    }

                }
            }

            network_mes = connection::receive_packet(&mut stream, &chat) => {
                match network_mes {
                    Ok(_) => {

                    }
                    Err(e) => {
                        println!("Connection lost: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
