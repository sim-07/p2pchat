mod handler;
mod network;
mod state;
mod ui;

use crate::handler::handle_packet::handle_packet;
use crate::network::listen::listen;
use crate::state::state_chat::{self, Chat, Connections, Member, Message};
use crate::state::state_packets::Packet;

use crate::network::connect_to::connect_to;
use crate::network::send::{self, send};
use crate::ui::handle_input::handle_input;

use clap::Parser;
use local_ip_address::local_ip;
use rand::random;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about = "P2P Chat", long_about = None)]
struct Cli {
    #[arg(short = 'c', long = "connect", num_args = 2, value_names = ["IP", "PORT"])]
    ip_param: Option<Vec<String>>,

    #[arg(short = 'q', long = "quit")]
    quit: bool,

    #[arg(short = 'u', long = "username")]
    username: Option<String>,

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

    let selected_port: u16 = args.listening_port;
    let connections: Connections = Connections::new();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), selected_port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    let used_port = listener.local_addr()?.port();
    let my_ip: String = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    println!("Your local ip: {}", my_ip);
    let username: String = args.username.unwrap_or_else(|| rand_username());

    let myself = Arc::new(Member::new(
        username.clone(),
        my_ip,
        used_port,
        Uuid::new_v4().to_string(),
    ));
    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    let myself_listen = Arc::clone(&myself);
    if let Some(params) = args.ip_param {
        let ip_to_connect: String = params[0].clone();
        let port_to_connect: u16 = params[1].parse().expect("Port must be a number");
        let myself_connect = Arc::clone(&myself);
        let chat = chat.clone();

        tokio::spawn(async move {
            let mut stream = match connect_to(&ip_to_connect, port_to_connect).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "Error connecting to {}:{}: {}",
                        ip_to_connect, port_to_connect, e
                    );
                    return;
                }
            };

            let packet_id = Packet::Identity((*myself_connect).clone(), true);
            if let Err(e) = send(&mut stream, &packet_id).await {
                eprintln!("Error sending identity: {}", e);
            }

            let packet_init = Packet::InitSyncRequest;
            if let Err(e) = send(&mut stream, &packet_init).await {
                eprintln!("Error sending init: {}", e);
            }

            let chat_clone = Arc::clone(&chat);
            let myself_in = Arc::clone(&myself_listen);

            tokio::spawn(listen_main(chat_clone, myself_in, stream));
        });
    } else {
        // Utente che starta la chat
        let chat = chat.clone();
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.expect("Failed to accept");
                let stream = stream;

                let chat_clone = Arc::clone(&chat);
                let myself_in = Arc::clone(&myself_listen);

                tokio::spawn(listen_main(chat_clone, myself_in, stream, connections));
            }
        });
    }    

    let chat_clone = Arc::clone(&chat);

    handle_input(chat_clone, myself, connections).await;

    Ok(())
}

fn rand_username() -> String {
    (0..4)
        .map(|_| (0x20u8 + (random::<f32>() * 96.0) as u8) as char)
        .collect()
}

async fn listen_main(chat: Arc<Mutex<Chat>>, myself: Arc<Member>, mut stream: TcpStream, connections: Connections) {
    let (mut reader, mut writer) = stream.into_split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Packet>();

    {
        let mut c_lock = connections.connections.lock().await;
        c_lock.push(tx);
    }

    tokio::spawn(async move {
        while let Some(packet) = rx.recv().await { // appena riceve il mess sul rx lo invia a tutti i membri della chat
            if let Err(_) = send(&mut writer, &packet).await {
                break; 
            }
        }
    });

    loop {
        if let Some(packet) = listen(&mut reader).await {
            handle_packet(packet, &chat, &*myself, tx.clone()).await;
        } else {
            break;
        }
    }
}