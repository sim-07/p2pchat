use std::collections::HashSet;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::send::send;
use crate::state::state_chat::Member;
use crate::state::state_packets::Packet;
use crate::state_chat::{self, Chat};
use crate::ui::handle_output::print_message;
use crate::ui::handle_output;

pub async fn handle_packet(
    packet: Packet,
    chat: &Arc<Mutex<Chat>>,
    myself: &Member,
    stream: &mut TcpStream,
) {
    match packet {
        Packet::UserMessage(message) => {
            let mut chat_lock = chat.lock().await;

            print_message(&message);
            chat_lock.add_message(message);
        }

        Packet::Sync(chat_received) => {
            let mut chat_lock = chat.lock().await;
            chat_lock.set_all_messages(chat_received.all_messages.clone());
            handle_output::print_all_messages(chat_received.all_messages);

            let diff: Vec<state_chat::Member> =
                get_members_diff(&chat_lock.members, &chat_received.members);

            for m in &diff {
                chat_lock.add_member(m.clone());

                let packet = Packet::Identity(myself.clone(), false);
                if let Err(e) = send(stream, &packet).await {
                    println!("Error sending identity Sync: {}", e);
                    return;
                }
            }
        }

        Packet::InitSyncRequest => {
            let chat_lock = chat.lock().await;
            let packet = Packet::Sync(chat_lock.clone());

            if let Err(e) = send(stream, &packet).await {
                println!("Connection error sending message: {}", e);
            }
        }

        Packet::Identity(new_member, idback) => {
            let mut chat_lock = chat.lock().await;
            chat_lock.add_member(new_member);

            let packet = Packet::Identity(myself.clone(), false);

            if idback {
                if let Err(e) = send(stream, &packet).await {
                    println!("Error sending identity Identity: {}", e);
                    return;
                }
            }
        }
    }
}

fn get_members_diff(m_loc: &Vec<Arc<Member>>, m_rec: &Vec<Arc<Member>>) -> Vec<state_chat::Member> {
    let loc_members: HashSet<_> = m_loc.iter().map(|m| &m.id).collect();

    m_rec
        .iter()
        .filter(|r| !loc_members.contains(&r.id))
        .map(|r| (**r).clone()) // (*r) dà Arc, (**r) dà Member.
        .collect()
}
