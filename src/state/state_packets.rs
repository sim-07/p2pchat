use serde::{Deserialize, Serialize};

use crate::state_chat::{Chat, Member, Message};

#[derive(Serialize, Deserialize)]
pub enum Packet {
    UserMessage(Message),
    InitSyncRequest,
    Sync(Chat),
    Identity(Member, bool)
}