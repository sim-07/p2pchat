use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message { // TODO trasferire anche file
    sender: String,
    text: String,
    timestamp: u64,
}

pub struct Member { // TODO creare nel main se stessi e aggiungersi a members. Poi inviare la nuova lista membri a tutti (aggiungere verifica legittimità)
    ip: String,
    port: u16,
    username: String
}

pub struct Chat {
    pub all_messages: HashMap<String, Vec<Arc<Message>>>,
    pub members: Vec<Member>,
}

impl Chat {
    pub fn new() -> Self {
        Self { all_messages: HashMap::new(), members: Vec::new() }
    }

    pub fn store_messages(&mut self, chat_id: String, sender: String, text: String) {
        let message = Message { sender, text, timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()};
        self.all_messages.entry(chat_id).or_insert_with(Vec::new).push(Arc::new(message));
    }

    pub fn get_all_local_messages(&self) -> HashMap<std::string::String, Vec<Arc<Message>>> {
        self.all_messages.clone()
    }
}

pub async fn manage_chat(chat: Arc<Mutex<Chat>>) {

    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    println!("--- Chat started ---");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let mut chat_lock = chat.lock().await;
                chat_lock.store_messages("general".to_string(), "Me".to_string(), line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting chat...");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

}