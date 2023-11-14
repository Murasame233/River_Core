use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::data::global::GLOBAL_NODE;

use super::{HandleRequest, Message, MessageData};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewNodeReq {
    id: Uuid,
    report_address: String,
}

impl NewNodeReq {
    pub fn new(id: Uuid, report_address: String) -> Self {
        Self { id, report_address }
    }
    pub async fn new_req() -> Message {
        let node = GLOBAL_NODE.read().await;
        let m = Message::new_request(
            String::from("NewNodeReq"),
            MessageData::NewNodeReq(Self::new(node.id, node.report_address.clone())),
        );
        m
    }
}
impl HandleRequest for NewNodeReq {
    async fn handle_request(&self, _req: &MessageData) -> MessageData {
        let node = GLOBAL_NODE.read().await;
        MessageData::NewNodeRes(NewNodeRes::new_res(node.id, node.report_address.clone()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewNodeRes {
    pub id: Uuid,
    pub report_address: String,
}

impl NewNodeRes {
    pub fn new_res(id: Uuid, report_address: String) -> Self {
        Self { id, report_address }
    }
}
