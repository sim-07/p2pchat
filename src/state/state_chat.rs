use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, broadcast};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub sender: Member,
    pub text: String,
    pub timestamp: u64,
}

impl Message {
    pub fn new(sender: Member, text: String, timestamp: u64) -> Self {
        Self {
            sender,
            text,
            timestamp,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Member {
    // TODO creare nel main se stessi e aggiungersi a members. Poi inviare la nuova lista membri a tutti (aggiungere verifica legittimità)
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub id: String,
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

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn set_all_messages(&mut self, messages: Vec<Arc<Message>>) {
        self.all_messages = messages;
    }

    pub fn set_members(&mut self, members: Vec<Arc<Member>>) {
        self.members = members;
    }

    pub fn add_message(&mut self, message: Message) {
        self.all_messages.push(Arc::new(message));
    }

    pub fn add_member(&mut self, member: Member) {
        self.members.push(Arc::new(member));
    }

    pub fn print_all_messages(&self) {
        for message in &self.all_messages {
            println!("[{:?}]: {}", message.sender, message.text);
        }
    }

    pub fn print_message(&self, message: &Message) {
        println!("[{:?}]: {}", message.sender.username, message.text);
    }
}

pub async fn start_chat(chat: Arc<Mutex<Chat>>, member: Arc<Mutex<Member>>, tx: broadcast::Sender<Message>) {
    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    println!("--- Chat started ---");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                //println!("Message: {}", line);
                let member_lock = member.lock().await.clone();
                let mut chat_lock = chat.lock().await;

                let message: Message = Message::new(member_lock, line, get_timestamp());
                chat_lock.add_message(message.clone());

                if let Err(e) = tx.send(message) { // invio il messaggio su tx
                    println!("No users connected ({})", e);
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