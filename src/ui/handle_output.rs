use std::io::{self, Write};
use crate::state::state_chat::Message;

pub fn print_all_messages(all_messages: Vec<std::sync::Arc<Message>>) {
    print!("\r\x1b[2K"); // \r sposta cursore all'inizio riga (prima di >>) e \x1b[2K cancella tutta la riga
    for message in all_messages {
        println!("[{}]: {}", message.sender, message.text);
    }
    print!(">> ");
    io::stdout().flush().unwrap();
}

pub fn print_message(message: &Message) {
    print!("\r\x1b[2K");
    println!("[{}]: {}", message.sender, message.text);
    print!(">> ");
    io::stdout().flush().unwrap();
}