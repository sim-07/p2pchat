mod connect_to;
mod listening;

use clap::Parser;
use std::error::Error;
use std::io;
use std::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short = 'c', long = "connectto", num_args = 2, value_names = ["IP", "PORT"])]
    ip_param: Option<Vec<String>>,

    #[arg(short = 'q', long = "quit")]
    quit: bool,

    #[arg(short = 's', long = "start")]
    start: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.quit {
        println!("DISCONNECTED");
        return Ok(());
    }

    if args.start {
        start_listening();
    }

    if let Some(params) = args.ip_param {
        connection(params);
    } else {
        println!("No action specified.");
    }

    Ok(())
}

fn connection(params: Vec<String>) {
    let ip = &params[0];
    let port: u16 = params[1].parse().expect("Port must be a number");
    println!("Connecting to {}:{}...", ip, port);

    if let Err(e) = connect_to::connect_to(ip, port) {
        println!("Failed to connect: {}", e);
    } else {
        println!("Connection established.");
    }
}

fn start_listening() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");
    println!("Waiting for connections on port 8080...");

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