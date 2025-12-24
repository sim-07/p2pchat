use crate::state::state_chat::Message;

pub fn print_all_messages(all_messages: Vec<Message>) {
    for message in all_messages {
        println!("[{:?}]: {}", message.sender, message.text);
    }
}

pub fn print_message(message: &Message) {
    println!("[{:?}]: {}", message.sender.username, message.text);
}
