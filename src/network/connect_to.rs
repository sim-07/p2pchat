use tokio::net::TcpStream;

pub async fn connect_to(ip: &str, port: u16) -> tokio::io::Result<TcpStream> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address).await {
        Ok(stream) => Ok(stream),
        Err(e) => {
            println!("Failed to connect to {}: {}", address, e);
            Err(e)
        }
    }
}