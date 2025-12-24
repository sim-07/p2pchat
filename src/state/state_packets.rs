use serde::{Deserialize, Serialize};

use crate::state_chat::{Chat, Member, Message};

#[derive(Serialize, Deserialize)]
pub enum Packet {
    UserMessage(Message),
    InitSyncRequest,
    Sync(Chat),
    Identity(Member)
}

// TODO prima di qualsiasi messaggio mi connetto e invio un IDENTITY con i miei dati, l'altro utente risponde coi propri dati. 
// All'inizio mi connetto ad un peer che mi invia lo stato globale. 
// Mi CONNETTO ad ogni membro che non conosco e gli invio IDENTITY, così l'altro può aggiungermi alla lista peer conosciuti. 