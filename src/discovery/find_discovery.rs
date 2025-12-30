use crate::state::state_discovery::DiscoveryPacket;

pub async fn find_discovery() {
    let disc_packet = DiscoveryPacket::Discovery;
    let bytes = serde_json::to_vec(&disc_packet).unwrap();

    let mult_addr = "239.255.42.99:9000".parse().unwrap();
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

    udp_socket.send_to(&bytes, mult_addr).await.unwrap();

}