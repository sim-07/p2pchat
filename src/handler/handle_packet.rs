use std::collections::HashSet;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use crate::network::connect_to::connect_to;
use crate::network::listen::listen;
use crate::send;
use crate::state::state_chat::Member;
use crate::state::state_packets::Packet;
use crate::state_chat::{self, Chat, Message};
use crate::ui::handle_output::print_message;

pub async fn handle_packet(packet: Packet, chat: &Arc<Mutex<Chat>>, myself: Arc<Mutex<Member>>) { // ricevere stream
    match packet {

        Packet::UserMessage(message) => {
            let mut chat_lock = chat.lock().await;

            print_message(&message);
            chat_lock.add_message(message);
        }

        Packet::Sync(chat_received) => {
            let mut chat_lock = chat.lock().await;
            chat_lock.set_all_messages(chat_received.all_messages.clone());

            // let mut chat_lock = chat.lock().await;
            // chat_lock.set_all_messages(chat_received.all_messages.clone());

            // let diff: Vec<state_chat::Member> = get_members_diff(&chat_lock, &chat_received);

            // println!("DIFF: {:?} ++++++++++++++++++++++++++++++++", diff);

            // for m in &diff {
            //     chat_lock.add_member(m.clone());
            // }

            // println!("-------------- MEMBERS -------------------");
            // println!("{:?}", chat_lock.members);
            // println!("------------------------------------------");

            // if diff.len() >= 1 {
            //     let chat_clone = Arc::clone(chat);
            //     tokio::spawn(async move {
            //         for m in diff {
            //             if let Ok(stream) = connect_to(&m.ip, m.port).await {
            //                 // PROBLEMA: server vede client come nuovo membro e allora si connette, creando doppia connessione
            //                 let chat_clone_in = Arc::clone(&chat_clone);
            //                 listen(stream, chat_clone_in, tx.clone());
            //             } else {
            //                 eprintln!("Failed to connect to {} at {}:{}", m.username, m.ip, m.port);
            //             }
            //         }
            //     });
            // }
        }

        Packet::InitSyncRequest => {
            // let mut chat_lock = chat.lock().await;
            // chat_lock.add_member(member);

            // let packet = Packet::Sync(chat_lock.clone());
            // if let Err(e) = send::send(stream, &packet).await {
            //     println!("Connection error sending message: {}", e);
            // }
        }

        Packet::Identity(new_member) => {
            let mut chat_lock = chat.lock().await;
            chat_lock.add_member(new_member);
        },
    }
}

fn get_members_diff(
    chat_lock: &tokio::sync::MutexGuard<'_, Chat>,
    chat_received: &Chat,
) -> Vec<state_chat::Member> {
    let loc_members: HashSet<_> = chat_lock.members.iter().map(|m| &m.id).collect();

    chat_received
        .members
        .iter()
        .filter(|r| !loc_members.contains(&r.id))
        .map(|r| (**r).clone()) // (*r) dà Arc, (**r) dà Member.
        .collect()
}
