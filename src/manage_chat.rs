use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    // TODO trasferire anche file
    sender: String,
    text: String,
    timestamp: u64,
}

#[derive(PartialEq, Clone)]
pub struct Member {
    // TODO creare nel main se stessi e aggiungersi a members. Poi inviare la nuova lista membri a tutti (aggiungere verifica legittimità)
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub id: String,
}

pub struct Chat {
    pub all_messages: Vec<Arc<Message>>,
    pub members: Vec<Arc<Member>>,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            all_messages: Vec::new(),
            members: Vec::new(),
        }
    }

    pub fn store_messages(&mut self, sender: String, text: String) {
        let message: Message = Message {
            sender,
            text,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.all_messages.push(Arc::new(message));
    }

    pub fn get_all_local_messages(&self) -> Vec<Arc<Message>> {
        self.all_messages.clone()
    }

    pub fn get_all_local_members(&self) -> Vec<Arc<Member>> {
        self.members.clone()
    }
}

impl Member {
    pub fn new(username: String, ip: String, port: u16, id: String) -> Self {
        Self {
            username,
            ip,
            port,
            id,
        }
    }
}

pub async fn add_member(chat: Arc<Mutex<Chat>>, member: Arc<Mutex<Member>>) {
    let member_data = member.lock().await.clone();

    let mut chat_lock = chat.lock().await;
    chat_lock.members.push(Arc::new(member_data));
}

pub async fn start_chat(chat: Arc<Mutex<Chat>>, member: Arc<Mutex<Member>>) {
    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    println!("--- Chat started ---");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let sender_username = member.lock().await.clone();
                let mut chat_lock = chat.lock().await;
                chat_lock.store_messages(sender_username.username, line);
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
