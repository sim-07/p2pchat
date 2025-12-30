use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum DiscoveryPacket {
    Discovery,
    DiscoveryRes(String, u16),
}