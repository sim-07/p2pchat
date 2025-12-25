use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state_chat::{Chat, Member, Message};

pub async fn start_chat(chat: Arc<Mutex<Chat>>, member: Arc<Member>, tx: broadcast::Sender<Message>) {
    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    println!("--- Chat started ---");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                //println!("Message: {}", line);
                let mut chat_lock = chat.lock().await;

                let message: Message = Message::new((*member).username.clone(), line, get_timestamp());
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