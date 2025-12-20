use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

struct Message {
    sender: String,
    text: String,
    timestamp: u64,
}

pub fn manage_chat(action: String) {

    println!("--- Chat started ---");
    let mut all_chat_messages: HashMap<String, Vec<Message>> = HashMap::new();

    match (
        "store_messages" => 
    )

}

// pub fn store_messages() {

// }

// pub fn get_all_messages() -> HashMap<String, Vec<Message>> {
    
// }