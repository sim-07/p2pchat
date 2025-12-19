use std::net::TcpStream;
use std::error::Error;

pub fn connect_to(ip: &str, port: u16) -> Result<TcpStream, Box<dyn Error>> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address) {
        Ok(stream) => {
            println!("Successfully connected to {}", address);
            Ok(stream)
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

