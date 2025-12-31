use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{
    discovery::handle_packet_discovery::handle_packet_discovery,
    state::state_discovery::DiscoveryPacket,
};

pub async fn listen_discovery(ip: String, port: u16, tx: mpsc::UnboundedSender<(String, u16)>, id: String) {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    socket.set_reuse_address(true).unwrap();

    #[cfg(not(windows))]
    socket.set_reuse_port(true).unwrap();
    
    socket.set_nonblocking(true).unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    if let Err(e) = socket.bind(&addr.into()) {
        println!("Error binding socket: {}", e);
    }

    let multicast_addr = Ipv4Addr::new(239, 255, 42, 99);
    let interface = Ipv4Addr::new(0, 0, 0, 0);
    if let Err(e) = socket.join_multicast_v4(&multicast_addr, &interface) {
        println!("Error joining multicast: {}", e);
    }

    let std_udp: std::net::UdpSocket = socket.into();
    let udp_socket: UdpSocket = UdpSocket::from_std(std_udp).unwrap();

    let mut buf = [0u8; 1024];
    loop {
        if let Ok((len, addr)) = udp_socket.recv_from(&mut buf).await {
            
            if let Ok(packet_rec) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                handle_packet_discovery(packet_rec, ip.clone(), port, &udp_socket, addr, tx.clone(), id.clone()).await;
            }
        }
    }
}
