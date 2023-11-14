use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::new_node::{NewNodeReq, NewNodeRes};

pub mod new_node;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Requst,
    Response,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageData {
    NewNodeReq(NewNodeReq),
    NewNodeRes(NewNodeRes),
    Text(String),
}

impl MessageData {
    pub async fn handle_request(&self) -> MessageData {
        if let MessageData::NewNodeReq(req) = self {
            return req.handle_request(self).await;
        } else {
            return MessageData::Text("Error".to_string());
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub name: String,
    pub message_type: MessageType,
    pub data: MessageData,
}
impl Message {
    pub fn new_request(name: String, data: MessageData) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name,
            message_type: MessageType::Requst,
            data,
        }
    }
    pub fn new_response(id: Uuid, name: String, data: MessageData) -> Self {
        Self {
            id,
            name,
            message_type: MessageType::Response,
            data,
        }
    }
    pub async fn handle_request(&self) -> Self {
        Self::new_response(self.id, self.name.clone(), self.data.handle_request().await)
    }
}

trait HandleRequest {
    async fn handle_request(&self, req: &MessageData) -> MessageData;
}