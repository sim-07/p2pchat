use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

use crate::{handler::handle_packet, state::state_discovery::DiscoveryPacket};

pub async fn listen_discovery(ip: String, port: u16) {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    let _ = socket.set_reuse_address(true); // indirizzo condiviso Linux/mac
    //socket.set_reuse_port(true);

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
    let udp_socket = UdpSocket::from_std(std_udp).unwrap();

    let mut buf = [0u8; 1024];
    loop {
        let (len, addr) = udp_socket.recv_from(&mut buf).await.unwrap();
        let packet_rec: DiscoveryPacket = serde_json::from_slice(&buf[..len]).unwrap();

        handle_packet_discovery(packet_rec, ip, port);

    }
}
