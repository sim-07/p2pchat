use tokio::{net::UdpSocket, sync::mpsc};

use crate::state::state_discovery::DiscoveryPacket;

pub async fn handle_packet_discovery(
    packet_rec: DiscoveryPacket,
    ip: String,
    port: u16,
    udp_socket: &UdpSocket,
    tx: mpsc::UnboundedSender<(String, u16)>,
    my_id: String,
) {
    match packet_rec {
        DiscoveryPacket::Discovery(id) => {

            if id != my_id {
                let reply = DiscoveryPacket::DiscoveryRes(ip, port, my_id, id);
                let reply_bytes = serde_json::to_vec(&reply).unwrap();

                let multicast_addr = "239.255.42.99:9000";

                if let Err(e) = udp_socket.send_to(&reply_bytes, multicast_addr).await {
                    println!("Error discovery: {}", e);
                }
            }
        }
        DiscoveryPacket::DiscoveryRes(ip_res, port_res, id_sender, rec_id) => {
            if id_sender != my_id && rec_id == my_id {
                if let Err(_e) = tx.send((ip_res, port_res)) {
                    //println!("Error DiscoveryRes: {}", e);
                }
            }
        }
    }
}
