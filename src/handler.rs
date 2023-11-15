use futures::SinkExt;
use futures::StreamExt;
use std::str::FromStr;
use std::{net::SocketAddr, sync::Arc};

use log::info;
use tokio::sync::oneshot;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::handshake::server::Response;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use uuid::Uuid;

use crate::data::global::StreamType;
use crate::data::global::GLOBAL_USER;
use crate::data::global::GLOBAL_USER_STREAM_TYPE;
use crate::data::global::{
    GLOBAL_HANDLE, GLOBAL_PEERS, GLOBAL_RECEIVER, GLOBAL_REQUEST_LOCKER, GLOBAL_SENDER,
    GLOBAL_USER_HANDLE, GLOBAL_USER_RECEIVER, GLOBAL_USER_SENDER,
};
use crate::data::peer::Peer;
use crate::data_protocol::{handle_user_messge, UserMessage};
use crate::peer_protocol::new_node::NewNodeReq;
use crate::peer_protocol::{Message as PeerMessage, MessageData, MessageType};

pub async fn handle_data_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    info!("Incoming TCP connection from: {}", addr);
    let (sender, receiver) = oneshot::channel::<String>();
    let callback = |req: &Request, response: Response| {
        info!("Handle the path");
        let p = req.uri().path().to_string();
        info!("The request's path is: {}", p);
        sender.send(p).unwrap();
        Ok(response)
    };
    let ws_stream =
        tokio_tungstenite::accept_hdr_async(MaybeTlsStream::Plain(raw_stream), callback)
            .await
            .expect("Error during the websocket handshake occurred");
    info!("wait For Path");
    let mut path = receiver.await.unwrap().clone();
    path = path.strip_prefix("/").unwrap().to_string();

    let uuid = Uuid::from_str(&path).expect("Invalid path");
    info!("WebSocket connection established: {}", addr);

    let account_lock = GLOBAL_USER
        .get(&uuid)
        .expect("invalid uuid")
        .get_mut()
        .clone();
    let account_type_lock = GLOBAL_USER_STREAM_TYPE
        .get(&uuid)
        .expect("invalid uuid")
        .get_mut()
        .clone();
    let father = account_lock.lock().await.father.clone();
    let cloned_stream_type = account_type_lock.lock().await.clone();
    let cloned_uuid = uuid.clone();

    let (write, read) = ws_stream.split();
    let shared_write = Arc::new(Mutex::new(write));
    let shared_read = Arc::new(Mutex::new(read));
    let shared_read_clone = shared_read.clone();
    let handle = tokio::spawn(async move {
        while let Some(msg) = shared_read_clone.lock().await.next().await {
            let msg = msg.unwrap();
            if msg.is_text() || msg.is_binary() {
                let text = msg.to_text().unwrap();
                let message: UserMessage = serde_json::from_str(text).unwrap();
                if cloned_stream_type == StreamType::Main {
                    let v = account_lock.lock().await.sub.clone().unwrap();
                    if message.from == cloned_uuid && v.contains(&message.to){
                        handle_user_messge(message).await;
                    }
                } else {
                    if message.from == cloned_uuid && message.to == father.unwrap() {
                        handle_user_messge(message).await;
                    }
                }
            }
        }
    });
    GLOBAL_USER_SENDER.insert(uuid, shared_write).unwrap();
    GLOBAL_USER_RECEIVER.insert(uuid, shared_read).unwrap();
    GLOBAL_USER_HANDLE
        .insert(uuid, Arc::new(Mutex::new(handle)))
        .unwrap();

    Ok(())
}

pub async fn handle_peer_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(MaybeTlsStream::Plain(raw_stream))
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

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
                } else if message.message_type == MessageType::Response {
                    let v = GLOBAL_REQUEST_LOCKER.get(&message.id).unwrap();
                    v.remove_entry().1.send(message).unwrap();
                } else if message.message_type == MessageType::Relay {
                    match message.data {
                        MessageData::UserMessage(message) => {
                            handle_user_messge(message).await;
                        }
                        _ => {
                            info!("Invalid relay")
                        }
                    }
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
