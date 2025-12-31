use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum DiscoveryPacket {
    Discovery(String),
    DiscoveryRes(String, u16),
}