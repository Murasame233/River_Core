use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use log::info;
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    data::{
        global::{
            GLOBAL_HANDLE, GLOBAL_NODE, GLOBAL_PEERS, GLOBAL_RECEIVER, GLOBAL_REQUEST_LOCKER,
            GLOBAL_SENDER,
        },
        peer::Peer,
    },
    peer_protocol::{new_node::NewNodeReq, Message as PeerMessage, MessageData, MessageType}, data_protocol::handle_user_messge,
};

pub async fn run() {
    let init_node = GLOBAL_NODE.read().await.init_node.clone();
    if let Some(target) = init_node {
        let connect_addr = String::from(target);
        let url = url::Url::parse(&connect_addr).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
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
    };
}
