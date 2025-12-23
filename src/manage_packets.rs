use serde::{Deserialize, Serialize};

use crate::manage_chat::{Chat, Message};

#[derive(Serialize, Deserialize)]
pub enum Packet {
    UserMessage(Message),
    InitSyncRequest,
    Sync(Chat), // TODO passare solo members
}