use std::net::TcpStream;
use std::io::Result;

pub fn connect_to(ip: &str, port: u16, chat_id: &str) -> Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address) {
        Ok(stream) => {
            println!("Successfully connected to {}", address);
            Ok(stream)
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub fn request_messages(chat_id: &str, stream: &mut TcpStream) -> Result<()> {
    // inviare richiesta avente tipo system per esempio per differenziare dai messaggi

    Ok(())
}