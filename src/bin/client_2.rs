use std::str::FromStr;

use algonaut::transaction::account::Account;
use data_encoding::BASE64;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use uuid::Uuid;

const MEMO2:&'static str = "crystal tiger wide simple cricket sick clerk cupboard master jump reflect level page solve sound depth jazz party wool wage ill window battle above reason";

#[derive(Deserialize, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Sign {
    pub address: String,
    pub msg: String,
    pub signature: String,
}

fn main() {
    // get user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let father = input.trim().to_string();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
        let (sub, url) = sub_stream(&father).await;

        let (stdin_tx, mut stdin_rx) = futures::channel::mpsc::unbounded();
        tokio::spawn(read_stdin(stdin_tx));

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (mut write, read) = ws_stream.split();
        let handle = tokio::spawn(async {
            // print all incomming;
            read.for_each(|msg| async {
                let message: UserMessage =
                    serde_json::from_str(msg.unwrap().to_string().as_str()).unwrap();
                let UserMessageData::Text(s) = message.data;
                println!("Other User Stream: {}", s);
            })
            .await;
        });

        println!("Now you can type");
        while let Some(s) = stdin_rx.next().await {
            let user_message: UserMessage = UserMessage::new_message(
                Uuid::from_str(&sub).unwrap(),
                Uuid::from_str(&father).unwrap(),
                s,
            );
            write
                .send(Message::Text(serde_json::to_string(&user_message).unwrap()))
                .await
                .unwrap();
        }
        let _ = handle.await;
    });
}

async fn sub_stream(father: &String) -> (String, Url) {
    let msg = father.clone();
    let account = Account::from_mnemonic(MEMO2).unwrap();
    let signature = BASE64.encode(&account.generate_sig(msg.as_bytes()).0);
    let sign = Sign {
        address: account.address().to_string(),
        msg,
        signature,
    };
    let client = reqwest::Client::new();
    let req = client.post("http://127.0.0.1:2003/join_stream").json(&sign);
    let re = req.send().await.unwrap();
    let sub = re.text().await.unwrap();
    let mut connect_addr = String::from("ws://127.0.0.1:2503/");
    connect_addr.push_str(&sub);
    let url = Url::parse(&connect_addr).unwrap();

    return (sub, url);
}

async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<String>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(String::from_utf8(buf).unwrap()).unwrap();
    }
}

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
