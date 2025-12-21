pub mod connection;
mod listening;
mod manage_chat;

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::manage_chat::Chat;

#[derive(Parser, Debug)]
#[command(author, version, about = "P2P Chat", long_about = None)]
struct Cli {
    #[arg(short = 'c', long = "connect", num_args = 3, value_names = ["IP", "PORT", "CHAT_ID"])]
    ip_param: Option<Vec<String>>,

    #[arg(short = 'q', long = "quit")]
    quit: bool,

    #[arg(short = 'u', long = "username")]
    username: String,

    #[arg(short = 'p', long = "port", value_parser = clap::value_parser!(u16).range(1..), default_value_t = 8080)]
    listening_port: u16,
}

// allow private chat
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.quit {
        println!("DISCONNECTED");
        return Ok(());
    }

    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    if let Some(params) = args.ip_param {
        let ip: &String = &params[0];
        let port: u16 = params[1].parse().expect("Port must be a number");
        let chat_id: &String = &params[2];

        let mut stream: Option<TcpStream> = connect(ip, port, chat_id).await;

        request_messages(chat_id, stream.as_mut());
    } else {
        println!("No action specified.");
    }

    let chat_listening = Arc::clone(&chat);
    tokio::spawn(async move {
        start_listening(args.listening_port, chat_listening).await;
    });

    
    manage_chat::manage_chat(Arc::clone(&chat));

    Ok(())
}

fn request_messages(chat_id: &String, stream: Option<&mut TcpStream>) {
    if let Some(stream) = stream {
        if let Err(e) = connection::request_messages(chat_id, stream) {
            println!("Error sending request: {}", e);
        }
    } else {
        println!("No TcpStream available");
    }
}

async fn connect(ip: &String, port: u16, chat_id: &String) -> Option<TcpStream> {
    println!("Connecting to {}:{}...", ip, port);

    match connection::connect_to(ip, port, chat_id).await {
        Ok(stream) => {
            println!("Connection established with {}:{}.", ip, port);
            return Some(stream);
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            return None;
        }
    };
}

async fn start_listening(listening_port: u16, chat: Arc<Mutex<Chat>>) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), listening_port);
    let listener = TcpListener::bind(&socket).await.expect("Failed to bind to address");

    println!("Listening on port {} for connections", listening_port);

    loop {
        let (stream, _) = listener.accept().await.expect("Failed to accept");

        let chat_clone = Arc::clone(&chat);

        tokio::spawn(async move {
            listening::listen(stream, chat_clone).await;
        });
    }
}
