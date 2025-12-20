pub mod connection;
mod listening;
mod manage_chat;

use clap::Parser;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.quit {
        println!("DISCONNECTED");
        return Ok(());
    }

    let mut chat: Chat = Chat::new();


    if let Some(params) = args.ip_param {
        let ip: &String = &params[0];
        let port: u16 = params[1].parse().expect("Port must be a number");
        let chat_id: &String = &params[2];

        let mut stream: Option<TcpStream> = connect(ip, port, chat_id);

        request_messages(chat_id, stream.as_mut());
    } else {
        println!("No action specified.");
    }

    start_listening(args.listening_port, &mut chat);
    manage_chat::manage_chat(&mut chat);

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

fn connect(ip: &String, port: u16, chat_id: &String) -> Option<TcpStream> {
    println!("Connecting to {}:{}...", ip, port);

    match connection::connect_to(ip, port, chat_id) {
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

fn start_listening(listening_port: u16, chat: &Chat) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), listening_port);
    let listener = TcpListener::bind(&socket).expect("Failed to bind to address");

    println!("Listening on port {} for connections", listening_port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| listening::listen(stream, &mut chat));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}
