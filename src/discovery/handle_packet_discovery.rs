use crate::state::state_discovery::DiscoveryPacket;

pub async fn handle_packet_discovery(packet: DiscoveryPacket, ip: String, port: u16) {
    match packet_rec {
            DiscoveryPacket::Discovery => {
                let reply = DiscoveryPacket::DiscoveryRes(ip, port);
                let reply_bytes = bincode::serialize(&reply).unwrap();

                udp_socket.send_to(&reply_bytes, addr).await.unwrap();
            },
            DiscoveryPacket::DiscoveryRes(ip, port)  => {
                // TODO prendere solo dati del primo peer che risponde e mandargli InitSyncRequest (implementare fallback)
            }
            _ => {}
        }
}