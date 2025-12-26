use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use crate::state::state_chat::Connections;
use crate::{
    state::state_packets::Packet,
    state_chat::{Chat, Member, Message},
};

pub async fn handle_input(
    chat: Arc<Mutex<Chat>>,
    member: Arc<Member>,
    connections: Connections,
) {
    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    println!("--- Chat started ---");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let mut chat_lock = chat.lock().await;

                let message: Message =
                    Message::new((*member).username.clone(), line, get_timestamp());
                chat_lock.add_message(message.clone());

                let packet: Packet = Packet::UserMessage(message);

                {
                    let conns = connections.connections.lock().await;
                    for c in conns.iter() {
                        let _ = c.send(packet.clone());
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting chat...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Timestamp error")
        .as_secs()
}
