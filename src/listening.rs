use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::manage_chat::{self, Chat, Member};

pub async fn listen(mut stream: TcpStream, chat: Arc<Mutex<Chat>>) {
    let peer_address = stream.peer_addr().expect("Could not get peer address");
    println!("{} has entered the chat", peer_address);

    loop {
        let mut buffer: [u8; 4096] = [0; 4096]; // Max amout of bytes from the client

        match stream.read(&mut buffer).await {
            Ok(0) => {
                // 0 bytes = connessione chiusa
                println!("{} disconnected", peer_address);
                break;
            }
            Ok(n) => {
                // quanto un peer si connette e manda qualcosa gli mando lo struct chat
                let received_data = String::from_utf8_lossy(&buffer[..n]); // TODO ricevo Member dal client (se stesso) e aggiorno la lista utenti della chat qui

                match serde_json::from_str::<Member>(&received_data) {
                    Ok(new_member) => {
                        println!("{} has connected", new_member.username);

                        manage_chat::add_member(Arc::clone(&chat), Arc::new(Mutex::new(new_member))).await;

                        let chat_lock = chat.lock().await;
                        let response_json =
                            serde_json::to_string(&*chat_lock).expect("Failed to serialize"); // *chat_lock per ottenere lo struct dal mutex, & per non prendere ownership

                        if let Err(e) = stream.write_all(response_json.as_bytes()).await {
                            eprintln!("Failed to write response: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("Member data not valid: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                return;
            }
        };
    }
}
