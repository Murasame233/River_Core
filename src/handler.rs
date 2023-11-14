use std::{net::SocketAddr, sync::Arc};

use futures::SinkExt;
use futures::StreamExt;

use tokio::sync::oneshot;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;

use crate::data::global::{
    GLOBAL_HANDLE, GLOBAL_PEERS, GLOBAL_RECEIVER, GLOBAL_REQUEST_LOCKER, GLOBAL_SENDER,
};
use crate::data::peer::Peer;
use crate::peer_protocol::new_node::NewNodeReq;
use crate::peer_protocol::{Message as PeerMessage, MessageData, MessageType};

pub async fn handle_api_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    println!("Incoming TCP connection from: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await.expect("Send Error");
        }
    }

    Ok(())
}

pub async fn handle_peer_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(MaybeTlsStream::Plain(raw_stream))
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (write, read) = ws_stream.split();
    let shared_write = Arc::new(Mutex::new(write));
    let shared_read = Arc::new(Mutex::new(read));
    let shared_write_clone = shared_write.clone();
    let shared_read_clone = shared_read.clone();
    let handle = tokio::spawn(async move {
        while let Some(msg) = shared_read_clone.lock().await.next().await {
            let msg = msg.unwrap();
            if msg.is_text() || msg.is_binary() {
                let text = msg.to_text().unwrap();
                let message: PeerMessage = serde_json::from_str(text).unwrap();
                if message.message_type == MessageType::Requst {
                    let re = message.handle_request().await;
                    shared_write_clone
                        .lock()
                        .await
                        .send(Message::text(serde_json::to_string(&re).unwrap()))
                        .await
                        .unwrap();
                } else {
                    let v = GLOBAL_REQUEST_LOCKER.get(&message.id).unwrap();
                    v.remove_entry().1.send(message).unwrap();
                }
            }
        }
    });

    let new_node_request = NewNodeReq::new_req().await;

    let (sender, receiver) = oneshot::channel::<PeerMessage>();
    GLOBAL_REQUEST_LOCKER
        .insert(new_node_request.id, sender)
        .unwrap();

    shared_write
        .lock()
        .await
        .send(Message::text(
            serde_json::to_string(&new_node_request).unwrap(),
        ))
        .await
        .unwrap();

    let re = receiver.await.unwrap();
    if let MessageData::NewNodeRes(data) = re.data {
        let id = data.id;
        GLOBAL_PEERS
            .insert(
                id.clone(),
                Arc::new(Mutex::new(Peer::new(data.report_address))),
            )
            .unwrap();
        GLOBAL_SENDER.insert(id.clone(), shared_write).unwrap();
        GLOBAL_RECEIVER.insert(id.clone(), shared_read).unwrap();
        GLOBAL_HANDLE
            .insert(id.clone(), Arc::new(Mutex::new(handle)))
            .unwrap();
    };

    Ok(())
}
