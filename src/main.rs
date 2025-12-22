pub mod connection;
mod listening;
mod manage_chat;

use clap::Parser;
use local_ip_address::local_ip;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

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

    #[arg(short = 'p', long = "port", default_value_t = 0)]
    listening_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.quit {
        println!("DISCONNECTED");
        return Ok(());
    }

    let my_ip: String = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    println!("Your local ip: {}", my_ip);

    let username = args.username;
    let selected_port: u16 = args.listening_port;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), selected_port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    let used_port = listener.local_addr()?.port(); // anche se l'utente non ha impostato nessuna porta con quella di default a 0 l'OS sceglierà una porta libera
    println!("Your port: {}", used_port);
    let member: Arc<Mutex<Member>> = Arc::new(Mutex::new(Member::new(
        username.clone(),
        my_ip,
        used_port,
        Uuid::new_v4().to_string(),
    )));
    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    if let Some(params) = args.ip_param {
        let member_clone = Arc::clone(&member);

        tokio::spawn(async move {
            let ip: String = params[0].clone();
            let port: u16 = params[1].parse().expect("Port must be a number");
            
            let member_lock = member_clone.lock().await;
            let member_string =
                serde_json::to_string(&*member_lock).expect("Failed to serialize member");

            let mut stream = match connect(&ip, port).await {
                Some(s) => s,
                None => {
                    println!("Connection error");
                    return;
                }
            };

            if let Err(e) = stream.write_all(member_string.as_bytes()).await {
                eprintln!("Failed to send handshake: {}", e);
                return;
            }

            if let Ok(messages) = receive_messages(stream).await {
                println!("MESSAGGI: {}", messages);
                // TODO displaymessages...
            } else {
                println!("Disconnected");
            }
        });
    }

    let chat_listening = Arc::clone(&chat);
    tokio::spawn(async move {
        start_listening(listener, chat_listening).await;
    });

    manage_chat::start_chat(Arc::clone(&chat), member).await;

    if args.file_send {
        //TODO manage_chat::manage_send_files().await;
    }

    Ok(())
}

async fn receive_messages(mut stream: TcpStream) -> tokio::io::Result<String> {
    loop {
        match connection::receive_all_messages(&mut stream).await {
            // con await si ferma finché non arriva qualcosa
            Ok(am) => {
                println!("MESSAGES: {}", am)
            }
            Err(e) => {
                println!("Disconnected: {}", e);
                return Err(e);
            }
        };
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

async fn start_listening(listener: TcpListener, chat: Arc<Mutex<Chat>>) {
    loop {
        let (stream, _) = listener.accept().await.expect("Failed to accept");

        let chat_clone = Arc::clone(&chat);

        tokio::spawn(async move {
            listening::listen(stream, chat_clone).await;
        });
    }
}
