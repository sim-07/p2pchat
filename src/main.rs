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
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

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

    let member_lock_s = member.lock().await.clone();
    {
        let mut chat_lock_s = chat.lock().await;
        chat_lock_s.add_member(member_lock_s); // aggiungo l'utente corrente alla lista membri
    }

    let chat_listening = Arc::clone(&chat);

    let tx_listen = tx.clone();
    tokio::spawn(async move {
        // server side
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let chat_listening_copy = Arc::clone(&chat_listening);

            let rx_listen = tx_listen.subscribe();

            tokio::spawn(async move {
                listening::listen(stream, chat_listening_copy, rx_listen).await;
            });
        }
    });

    let tx_connect = tx.clone();
    if let Some(params) = args.ip_param {
        // client side
        let member_clone: Arc<Mutex<Member>> = Arc::clone(&member);
        let chat: Arc<Mutex<Chat>> = Arc::clone(&chat);

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
            let packet_handshake: Packet = Packet::InitSyncRequest(member_to_send);
            if let Err(e) = send::send(&mut stream, &packet_handshake).await {
                println!("Error sending messages: {}", e);
            }

            if let Err(e) = connection::receive_packet(&mut stream, &chat).await {
                println!("Error receiving packets: {}", e);
            }

            let rx_listen = tx_connect.subscribe();
            listening::listen(stream, chat, rx_listen).await;
        });
    }

    manage_chat::start_chat(Arc::clone(&chat), member, tx.clone()).await;

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
