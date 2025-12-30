use std::sync::Arc;

use crate::{
    handler::handle_packet::handle_packet,
    network::send::send,
    state::{
        state_chat::{Chat, Connections, Member},
        state_packets::Packet,
    },
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{Mutex, mpsc},
};

pub async fn get_packet(reader: &mut BufReader<OwnedReadHalf>) -> Option<Packet> {
    let mut line: String = String::new();

    match reader.read_line(&mut line).await {
        // legge fino a \n che ho messo in send per delimitare i messaggi
        Ok(0) => {
            println!("Peer disconnected"); // TODO aggiornare lista membri
            None
        }
        Ok(_) => match serde_json::from_str::<Packet>(line.trim()) {
            Ok(packet) => Some(packet),
            Err(e) => {
                eprintln!("Errore deserializzando pacchetto: {}", e);
                eprintln!("Contenuto ricevuto: {:?}", line);
                None
            }
        },
        Err(e) => {
            eprintln!("Stream error: {}", e);
            None
        }
    }
}

pub async fn listen_main(
    chat: Arc<Mutex<Chat>>,
    myself: Arc<Member>,
    reader: OwnedReadHalf,
    mut writer: OwnedWriteHalf,
    connections: Connections,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Packet>();

    {
        let mut c_lock = connections.connections.lock().await;
        c_lock.push(tx.clone());
    }

    tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            // appena riceve il mess sul rx lo invia a tutti i membri della chat
            if let Err(_) = send(&mut writer, &packet).await {
                break;
            }
        }
    });

    let mut buf_reader = BufReader::new(reader);
    loop {
        match get_packet(&mut buf_reader).await {
            Some(packet) => {
                handle_packet(packet, &chat, &*myself, tx.clone(), connections.clone()).await;
            }
            None => break,
        }
    }
}
