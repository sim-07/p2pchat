pub mod connection;
mod listening;
mod manage_chat;

use clap::Parser;
use uuid::Uuid;
use std::error::Error;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use local_ip_address::local_ip;

use crate::manage_chat::{Chat, Member};

#[derive(Parser, Debug)]
#[command(author, version, about = "P2P Chat", long_about = None)]
struct Cli {
    #[arg(short = 'c', long = "connect", num_args = 2, value_names = ["IP", "PORT"])]
    ip_param: Option<Vec<String>>,

    #[arg(short = 'q', long = "quit")]
    quit: bool,

    #[arg(short = 'u', long = "username")]
    username: String,

    #[arg(short = 'i', long = "initchat")]
    chat_init: bool,

    #[arg(short = 'f', long = "filesend")]
    file_send: bool,

    #[arg(short = 'p', long = "port", value_parser = clap::value_parser!(u16).range(1..), default_value_t = 8080)]
    listening_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.quit {
        println!("DISCONNECTED");
        return Ok(());
    }

    let my_ip: String = local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "127.0.0.1".to_string());
    println!("Your local ip: {}", my_ip);

    let username = args.username;
    let my_port: u16 = args.listening_port;
    let member: Arc<Mutex<Member>> = Arc::new(Mutex::new(Member::new(username.clone(), my_ip, my_port, Uuid::new_v4().to_string())));
    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    if let Some(params) = args.ip_param {
        let ip: &String = &params[0];
        let port: u16 = params[1].parse().expect("Port must be a number");

        let mut stream: Option<TcpStream> = connect(ip, port).await;

        receive_all_messages(stream.as_mut());
    }

    let chat_listening = Arc::clone(&chat);
    tokio::spawn(async move {
        start_listening(args.listening_port, chat_listening).await;
    });

    manage_chat::start_chat(Arc::clone(&chat), member).await;

    if args.file_send {
        //manage_chat::manage_send_files().await;
    }

    Ok(())
}

fn receive_all_messages(stream: Option<&mut TcpStream>) {
    if let Some(stream) = stream {
        let messages = connection::receive_all_messages(stream); /////////
        
    } else {
        println!("No TcpStream available");
    }
}

async fn connect(ip: &String, port: u16) -> Option<TcpStream> {
    println!("Connecting to {}:{}...", ip, port);

    match connection::connect_to(ip, port).await {
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
    let listener = TcpListener::bind(&socket)
        .await
        .expect("Failed to bind to address");

    println!("Listening on port {} for connections", listening_port);

    loop {
        let (stream, _) = listener.accept().await.expect("Failed to accept");

        let chat_clone = Arc::clone(&chat);

        tokio::spawn(async move {
            listening::listen(stream, chat_clone).await;
        });
    }
}
