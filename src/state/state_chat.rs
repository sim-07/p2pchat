use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

impl Message {
    pub fn new(sender: String, text: String, timestamp: u64) -> Self {
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

    pub fn add_message(&mut self, message: Message) {
        self.all_messages.push(Arc::new(message));
    }

    pub fn add_member(&mut self, member: Member) {
        self.members.push(Arc::new(member));
    }

}

