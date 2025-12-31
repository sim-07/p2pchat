use crate::state::state_discovery::DiscoveryPacket;
use tokio::net::UdpSocket;

pub async fn find_discovery(id: String) {
    let disc_packet = DiscoveryPacket::Discovery(id);
    let bytes = serde_json::to_vec(&disc_packet).unwrap();

    let mult_addr = "239.255.42.99:9000";

    let udp_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

    for _i in 0..3 {
        udp_socket.set_broadcast(true).unwrap();
        udp_socket.send_to(&bytes, mult_addr).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
