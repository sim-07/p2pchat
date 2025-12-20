use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Serialize, Deserialize)]
pub struct Message { // trasferire anche file
    sender: String,
    text: String,
    timestamp: u64,
}

pub struct Member { // creare nel main se stessi e aggiungersi a members. Poi inviare la nuova lista membri a tutti (aggiungere verifica legittimità)
    ip: String,
    port: u16,
    username: String
}

pub struct Chat {
    pub all_messages: HashMap<String, Vec<Message>>,
    pub members: Vec<Member>,
}

impl Chat {
    pub fn new() -> Self {
        Self { all_messages: HashMap::new(), members: Vec::new() }
    }

    pub fn store_messages(&mut self, chat_id: String, sender: String, text: String) {
        let message = Message { sender, text, timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()};
        self.all_messages.entry(chat_id).or_insert_with(Vec::new).push(message);
    }

    pub fn get_all_local_messages(&mut self) -> &HashMap<std::string::String, Vec<Message>> {
        &self.all_messages
    }
}

pub fn manage_chat(chat: &mut Chat) {

    println!("--- Chat started ---");

    let mut all_mess = chat.get_all_local_messages();

}