use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

use crate::network::connect_to::connect_to;
use crate::network::listen::listen_main;
use crate::network::send::send;
use crate::state::state_chat::{Connections, Member};
use crate::state::state_packets::Packet;
use crate::state_chat::{self, Chat};
use crate::ui::handle_output;
use crate::ui::handle_output::print_message;

pub async fn handle_packet(
    packet: Packet,
    chat: &Arc<Mutex<Chat>>,
    myself: &Member,
    tx: UnboundedSender<Packet>,
    connections: Connections,
) {
    match packet {
        Packet::UserMessage(message) => {
            let mut chat_lock = chat.lock().await;

            print_message(&message);
            chat_lock.add_message(message);
        }
        Packet::Sync(chat_received) => {
            let diff: Vec<state_chat::Member>;
            {
                let mut chat_lock = chat.lock().await;
                chat_lock.set_all_messages(chat_received.all_messages.clone());
                handle_output::print_all_messages(chat_received.all_messages);

                diff = get_members_diff(&chat_lock.members, &chat_received.members);

                for m in diff.clone() {
                    chat_lock.add_member(m.clone());
                }
            }

            for m in diff.clone() {
                let chat_clone = Arc::clone(chat);
                let myself_clone = Arc::new(myself.clone());
                let conns_clone = connections.clone();

                let packet = Packet::Identity(myself.clone(), false);

                conn(m, chat_clone, myself_clone, conns_clone, packet);
            }
        }
        Packet::InitSyncRequest => {
            let chat_lock = chat.lock().await;
            let packet = Packet::Sync(chat_lock.clone());

            if let Err(e) = tx.send(packet) {
                println!("Connection error in InitSyncRequest: {}", e);
                return;
            }
        }
        Packet::Identity(new_member, idback) => {
            {
                let mut chat_lock = chat.lock().await;

                if chat_lock.members.iter().any(|m| m.id == new_member.id) {
                    println!("Ricevuto membro già presente in members: {:?}", new_member);
                } else {
                    chat_lock.add_member(new_member);
                }
            }

            let packet = Packet::Identity(myself.clone(), false);

            if idback == true {
                if let Err(e) = tx.send(packet) {
                    println!("Connection error in Identity: {}", e);
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

fn conn(
    m: Member,
    chat_clone: Arc<Mutex<Chat>>,
    myself_clone: Arc<Member>,
    conns_clone: Connections,
    packet: Packet,
) {
    tokio::spawn(async move {
        match connect_to(&m.ip, m.port).await {
            Ok(stream) => {
                let (reader, mut writer) = stream.into_split();
                if let Err(e) = send(&mut writer, &packet).await {
                    println!("Error sending identity: {}", e);
                }
                listen_main(chat_clone, myself_clone, reader, writer, conns_clone).await;
            }
            Err(e) => {
                println!("Problem connect_to in Sync: {}", e);
                return;
            }
        }
    });
}
