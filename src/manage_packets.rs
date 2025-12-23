use serde::{Deserialize, Serialize};

use crate::manage_chat::{Chat, Member, Message};

#[derive(Serialize, Deserialize)]
pub enum Packet {
    UserMessage(Message),
    InitSyncRequest(Member),
    Sync(Chat),
}