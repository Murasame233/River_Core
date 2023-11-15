use futures::{executor::block_on, SinkExt};
use log::info;
use scc::HashSet;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    data::global::{GLOBAL_SENDER, GLOBAL_USER, GLOBAL_USER_SENDER},
    peer_protocol::Message as PeerMessage,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum UserMessageType {
    REG,
    MESSAGE,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserMessageData {
    Text(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMessage {
    pub id: Uuid,
    pub name: String,
    pub message_type: UserMessageType,
    pub data: UserMessageData,
    pub from: Uuid,
    pub to: Uuid,
    pub signature: String,
}
impl UserMessage {
    pub fn new_reg(from: Uuid, to: Uuid) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name: String::from("Reg"),
            message_type: UserMessageType::REG,
            data: UserMessageData::Text(String::new()),
            from,
            to,
            signature: String::new(),
        }
    }
    pub fn new_message(from: Uuid, to: Uuid, data: String) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name: String::from("message"),
            message_type: UserMessageType::MESSAGE,
            data: UserMessageData::Text(data),
            from,
            to,
            signature: String::new(),
        }
    }
}

lazy_static! {
    pub static ref HANDLED: HashSet<Uuid> = HashSet::new();
}

pub async fn find_or_broadcast(user_message: UserMessage) {
    let to = user_message.to;
    let finded = GLOBAL_USER_SENDER.get(&to).is_some();
    if finded {
        let lock = GLOBAL_USER_SENDER.get(&to).unwrap().get_mut().clone();
        lock.lock()
            .await
            .send(Message::Text(serde_json::to_string(&user_message).unwrap()))
            .await
            .unwrap();
    } else {
        let relay = PeerMessage::new_relay("relay".to_string(), user_message);
        GLOBAL_SENDER.scan(|_k, v| {
            block_on(async {
                v.lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&relay).unwrap()))
                    .await
                    .unwrap();
            })
        })
    }
}

pub async fn handle_user_messge(user_message: UserMessage) {
    let id = user_message.id;
    if HANDLED.contains(&id) {
        return;
    };
    HANDLED.insert(id).unwrap();
    info!("Received User message, from: {} to: {}",user_message.from,user_message.to);
    if user_message.message_type == UserMessageType::REG {
        let finded = GLOBAL_USER
            .get_async(&user_message.to)
            .await
            .is_some();
        if finded {
            let f = GLOBAL_USER
            .get_async(&user_message.to)
            .await
            .unwrap().get_mut().clone();
            let mut l = f.lock().await;
            l.sub.as_mut().unwrap().push(user_message.from.clone());
        }
    }
    find_or_broadcast(user_message).await;
}

pub async fn broad_cast_join(from: Uuid, father: Uuid) {
    let finded = GLOBAL_USER.get_async(&father).await.is_some();
    if finded {
        let arc = GLOBAL_USER.get_async(&father).await.unwrap().get_mut().clone();
        arc.lock().await.sub.as_mut().unwrap().push(from);
    } else {
        let user_message = UserMessage::new_reg(from, father);
        handle_user_messge(user_message).await;
    }
}
