use std::net::SocketAddr;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::state::state_discovery::DiscoveryPacket;

pub async fn handle_packet_discovery(
    packet_rec: DiscoveryPacket,
    ip: String,
    port: u16,
    udp_socket: &UdpSocket,
    addr: SocketAddr,
    tx: mpsc::UnboundedSender<(String, u16)>,
    my_id: String,
) {
    match packet_rec {
        DiscoveryPacket::Discovery(id) => {
            println!("Received Discovery");

            if id != my_id {
                let reply = DiscoveryPacket::DiscoveryRes(ip, port);
                let reply_bytes = serde_json::to_vec(&reply).unwrap();

                if let Err(e) = udp_socket.send_to(&reply_bytes, addr).await {
                    println!("Error discovery: {}", e);
                }
            }
        }
        DiscoveryPacket::DiscoveryRes(ip_res, port_res) => {
            println!("Received DiscoveryRes");
            if let Err(e) = tx.send((ip_res, port_res)) {
                println!("Error DiscoveryRes: {}", e);
            }
        }
    }
}
