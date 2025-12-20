mod connect_to;
mod listening;
mod manage_chat;

use clap::Parser;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

#[derive(Parser, Debug)]
#[command(author, version, about = "P2P Chat", long_about = None)]
struct Cli {
    #[arg(short = 'c', long = "connect", num_args = 2, value_names = ["IP", "PORT"])]
    ip_param: Option<Vec<String>>,

    #[arg(short = 'q', long = "quit")]
    quit: bool,

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

    if let Some(params) = args.ip_param {
        connection(params);
        
    } else {
        println!("No action specified.");
    }

    start_listening(args.listening_port);
    manage_chat::manage_chat("get_all_messages".to_string());

    Ok(())
}

fn connection(params: Vec<String>) {
    let ip = &params[0];
    let port: u16 = params[1].parse().expect("Port must be a number");
    println!("Connecting to {}:{}...", ip, port);

    if let Err(e) = connect_to::connect_to(ip, port) {
        println!("Failed to connect: {}", e);
    } else {
        println!("Connection established with {}:{}.", ip, port);
    }
}

fn start_listening(listening_port: u16) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), listening_port);
    let listener = TcpListener::bind(&socket).expect("Failed to bind to address");

    println!("Listening on port {} for connections", listening_port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| listening::listen(stream));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}