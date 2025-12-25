use std::sync::Arc;

use crate::state::state_chat::Message;

pub fn print_all_messages(all_messages: Vec<Arc<Message>>) {
    for message in all_messages {
        println!("[{:?}]: {}", message.sender, message.text);
    }
}

pub fn print_message(message: &Message) {
    println!("[{:?}]: {}", message.sender, message.text);
}
