pub mod connection;
mod listen;
mod manage_chat;
mod manage_packets;
mod send;

use clap::Parser;
use local_ip_address::local_ip;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

use crate::connection::connect_to;
use crate::manage_chat::{Chat, Member, Message};
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

    let (tx, _rx) = broadcast::channel::<Message>(32);

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

    let myself = member.lock().await.clone();
    {
        let mut chat_lock_s = chat.lock().await;
        chat_lock_s.add_member(myself); // aggiungo me stesso alla lista membri
    }

    let chat_listening = Arc::clone(&chat);

    let tx_clone_server = tx.clone();
    tokio::spawn(async move {
        // server side
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let chat_listening_copy = Arc::clone(&chat_listening);

            listen::listen(stream, chat_listening_copy, tx_clone_server.clone());
        }
    });

    if let Some(params) = args.ip_param {
        // client side
        let chat: Arc<Mutex<Chat>> = Arc::clone(&chat);

        let ip: String = params[0].clone();
        let port: u16 = params[1].parse().expect("Port must be a number");

        let tx_clone_client = tx.clone();
        let tx_clone_listen = tx.clone();
        tokio::spawn(async move {
            let mut stream = match connect_to(&ip, port).await {
                Ok(s) => s,
                Err(e) => {
                    println!("Connection error: {}", e);
                    return;
                }
            };

            let packet_handshake: Packet = Packet::InitSyncRequest;
            if let Err(e) = send::send(&mut stream, &packet_handshake).await {
                println!("Error sending messages: {}", e);
                return;
            }

            if let Err(e) = connection::receive_packet(&mut stream, &chat, tx_clone_client).await {
                println!("Error receiving packets: {}", e);
            }

            {
                let chat_lock = chat.lock().await;
                    let packet = manage_packets::Packet::Sync((*chat_lock).clone());

                    if let Err(e) = send::send(&mut stream, &packet).await {
                        println!("Error sending sync: {}", e);
                    }
            }

            listen::listen(stream, chat, tx_clone_listen);
        });
    }

    manage_chat::start_chat(Arc::clone(&chat), member, tx.clone()).await;

    if args.file_send {
        //TODO manage_chat::manage_send_files().await;
    }

    Ok(())
}