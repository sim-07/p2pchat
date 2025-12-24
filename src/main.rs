mod handler;
mod network;
mod state;
mod ui;

use crate::handler::handle_packet::handle_packet;
use crate::network::listen::listen;
use crate::state::state_chat::{self, Chat, Member, Message};
use crate::state::state_packets::Packet;

use crate::network::connect_to::connect_to;
use crate::network::send;
use crate::ui::handle_input;

use clap::Parser;
use local_ip_address::local_ip;
use rand::random;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), selected_port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    let used_port = listener.local_addr()?.port();
    let my_ip: String = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    println!("Your local ip: {}", my_ip);
    let username: String = args.username.unwrap_or_else(|| rand_username());

    let myself: Arc<Mutex<Member>> = Arc::new(Mutex::new(Member::new(
        username.clone(),
        my_ip,
        used_port,
        Uuid::new_v4().to_string(),
    )));
    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let chat_clone = Arc::clone(&chat);
            let myself_clone = myself.clone();

            tokio::spawn(async move {
                if let Some(packet) = listen(stream).await {
                    handle_packet(packet, &chat_clone, myself_clone).await;
                }
            });
        }
    });

    // let args = Cli::parse();

    // if args.quit {
    //     println!("DISCONNECTED");
    //     return Ok(());
    // }

    // let my_ip: String = local_ip()
    //     .map(|ip| ip.to_string())
    //     .unwrap_or_else(|_| "127.0.0.1".to_string());
    // println!("Your local ip: {}", my_ip);

    // let (tx, _rx) = broadcast::channel::<Message>(32);

    // let username = args.username;
    // let selected_port: u16 = args.listening_port;
    // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), selected_port);
    // let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    // let used_port = listener.local_addr()?.port(); // anche se l'utente non ha impostato nessuna porta con quella di default a 0 l'OS sceglierà una porta libera
    // println!("Your port: {}", used_port);
    // let member: Arc<Mutex<Member>> = Arc::new(Mutex::new(Member::new(
    //     username.clone(),
    //     my_ip,
    //     used_port,
    //     Uuid::new_v4().to_string(),
    // )));

    // let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    // let myself = member.lock().await.clone();
    // {
    //     let mut chat_lock_s = chat.lock().await;
    //     chat_lock_s.add_member(myself.clone()); // aggiungo me stesso alla lista membri
    // }

    // let chat_listening = Arc::clone(&chat);

    // let tx_clone_server = tx.clone();
    // tokio::spawn(async move {
    //     // server side
    //     loop {
    //         let (stream, _) = listener.accept().await.expect("Failed to accept");
    //         let chat_listening_copy = Arc::clone(&chat_listening);

    //         listen::listen(stream, chat_listening_copy, tx_clone_server.clone());
    //     }
    // });

    // if let Some(params) = args.ip_param {
    //     // client side
    //     let chat: Arc<Mutex<Chat>> = Arc::clone(&chat);

    //     let ip: String = params[0].clone();
    //     let port: u16 = params[1].parse().expect("Port must be a number");

    //     let tx_clone_client = tx.clone();
    //     let tx_clone_listen = tx.clone();

    //     let myself_clone = myself.clone();
    //     tokio::spawn(async move {
    //         let mut stream = match connect_to(&ip, port).await {
    //             Ok(s) => s,
    //             Err(e) => {
    //                 println!("Connection error: {}", e);
    //                 return;
    //             }
    //         };

    //         let packet_handshake: Packet = Packet::InitSyncRequest(myself_clone);
    //         if let Err(e) = send::send(&mut stream, &packet_handshake).await {
    //             println!("Error sending messages: {}", e);
    //             return;
    //         }

    //         if let Err(e) = connection::receive_packet(&mut stream, &chat, tx_clone_client).await {
    //             println!("Error receiving packets: {}", e);
    //         }

    //         listen::listen(stream, chat, tx_clone_listen);
    //     });
    // }

    // manage_chat::start_chat(Arc::clone(&chat), member, tx.clone()).await;

    // if args.file_send {
    //     //TODO manage_chat::manage_send_files().await;
    // }

    Ok(())
}

fn rand_username() -> String {
    (0..4)
        .map(|_| (0x20u8 + (random::<f32>() * 96.0) as u8) as char)
        .collect()
}
