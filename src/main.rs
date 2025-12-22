pub mod connection;
mod listening;
mod manage_chat;
mod manage_packets;
mod send;

use clap::Parser;
use local_ip_address::local_ip;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::manage_chat::{Chat, Member};
use crate::manage_packets::Packet;

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
    let chat_listening = Arc::clone(&chat);

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let chat_listening_copy = Arc::clone(&chat_listening);

            tokio::spawn(async move {
                listening::listen(stream, chat_listening_copy).await;
            });
        }
    });

    if let Some(params) = args.ip_param {
        let member_clone: Arc<Mutex<Member>> = Arc::clone(&member);
        let chat_sync: Arc<Mutex<Chat>> = Arc::clone(&chat);

        let ip: String = params[0].clone();
        let port: u16 = params[1].parse().expect("Port must be a number");

        tokio::spawn(async move {
            let mut stream = match connect(&ip, port).await {
                Some(s) => s,
                None => {
                    println!("Connection error");
                    return;
                }
            };

            let member_to_send = member_clone.lock().await.clone();

            let packet_handshake: Packet = Packet::InitRequest(member_to_send);
            send::send(&mut stream, &packet_handshake).await;

            if let Ok(chat) = connection::receive_init_chat(&mut stream).await {
                println!("Received messages");

                let mut lock = chat_sync.lock().await;
                *lock = chat;
                lock.print_all_messages();
            } else {
                println!("Disconnected");
            }
        });
    }

    manage_chat::start_chat(Arc::clone(&chat), member).await;

    if args.file_send {
        //TODO manage_chat::manage_send_files().await;
    }

    Ok(())
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
