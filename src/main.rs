mod discovery;
mod handler;
mod network;
mod state;
mod ui;

use crate::discovery::find_discovery::find_discovery;
use crate::discovery::listen_discovery::listen_discovery;
use crate::network::listen::listen_main;
use crate::state::state_chat::{self, Chat, Connections, Member};
use crate::state::state_packets::Packet;

use crate::network::connect_to::connect_to;
use crate::network::send::send;
use crate::ui::handle_input::handle_input;

use clap::Parser;
use local_ip_address::local_ip;
use rand::random;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
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

    #[arg(short = 'd', long = "discovery")]
    discovery: bool,

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
    println!("Your port: {}", used_port);
    let username: String = args.username.unwrap_or_else(|| rand_username());

    let myself = Arc::new(Member::new(
        username.clone(),
        my_ip.clone(),
        used_port,
        Uuid::new_v4().to_string(),
    ));
    let chat: Arc<Mutex<Chat>> = Arc::new(Mutex::new(Chat::new()));

    {
        let mut chat_lock = chat.lock().await;
        chat_lock.add_member((*myself).clone());
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<(String, u16)>();

    tokio::spawn(listen_discovery(
        my_ip.clone(),
        used_port,
        tx,
        (*myself.id).to_string(),
    ));

    if args.discovery {
        println!("Searching for other peers...");
        tokio::spawn(find_discovery((*myself.id).to_string()));

        let myself_dis = Arc::clone(&myself);
        let chat_dis = Arc::clone(&chat);
        let conn_dis = connections.clone();

        tokio::spawn(async move {
            while let Some((ip, port)) = rx.recv().await {
                match connection_main(
                    ip.clone(),
                    port,
                    myself_dis.clone(),
                    chat_dis.clone(),
                    conn_dis.clone(),
                )
                .await
                {
                    Ok(_) => {
                        println!("Connected to {}:{}", ip, port);
                        break; // Se si connette correttamente esco dal ciclo, altrimenti continuo a provare
                    }
                    Err(_) => {
                        println!(
                            "Failed to connect to {}:{}. Trying with next peer...",
                            ip, port
                        );
                        continue;
                    }
                }
            }
        });
    }

    // CONNECT
    if let Some(params) = args.ip_param {
        let ip_to_connect: String = params[0].clone();
        let port_to_connect: u16 = params[1].parse().expect("Port must be a number");
        let myself_connect = Arc::clone(&myself);
        let chat = chat.clone();
        let conn_clone = connections.clone();

        tokio::spawn(async move {
            if let Err(e) = connection_main(
                ip_to_connect.clone(),
                port_to_connect,
                myself_connect,
                chat,
                conn_clone,
            )
            .await
            {
                println!(
                    "Error connecting to {}:{}: {}",
                    ip_to_connect, port_to_connect, e
                );
            }
        });
    }

    // LISTEN
    let chat_clone: Arc<Mutex<Chat>> = Arc::clone(&chat);
    let conn_clone: Connections = connections.clone();
    let myself_clone: Arc<Member> = Arc::clone(&myself);

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");

            let stream = stream;

            let chat_clone = Arc::clone(&chat_clone);
            let myself_in = Arc::clone(&myself_clone);
            let conn_clone = conn_clone.clone();

            let (reader, writer) = stream.into_split();
            tokio::spawn(listen_main(
                chat_clone, myself_in, reader, writer, conn_clone,
            ));
        }
    });

    let chat_clone = Arc::clone(&chat);
    handle_input(chat_clone, myself, connections.clone()).await;

    Ok(())
}

fn rand_username() -> String {
    (0..4)
        .map(|_| (0x20u8 + (random::<f32>() * 96.0) as u8) as char)
        .collect()
}

async fn connection_main(
    ip_to_connect: String,
    port_to_connect: u16,
    myself: Arc<Member>,
    chat: Arc<Mutex<Chat>>,
    conn_clone: Connections,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream = connect_to(&ip_to_connect, port_to_connect).await?;

    let (reader, mut writer) = stream.into_split();

    let packet_id = Packet::Identity((*myself).clone(), true);
    if let Err(e) = send(&mut writer, &packet_id).await {
        println!("Error sending identity: {}", e);
    }

    let packet_init = Packet::InitSyncRequest;
    if let Err(e) = send(&mut writer, &packet_init).await {
        println!("Error sending init: {}", e);
    }

    let chat_clone = Arc::clone(&chat);
    let myself_in = Arc::clone(&myself);

    tokio::spawn(listen_main(
        chat_clone, myself_in, reader, writer, conn_clone,
    ));

    Ok(())
}
