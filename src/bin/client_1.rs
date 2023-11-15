use std::{str::FromStr, sync::Arc};

use algonaut::transaction::account::Account;
use data_encoding::BASE64;
use futures::{executor::block_on, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, sync::Mutex};
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use uuid::Uuid;

const MEMO:&'static str = "stereo room brief lion actual dress sister elephant cricket age mule elite month coil devote sun deputy vacant great reduce faint response hip abandon recycle";

#[derive(Deserialize, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Sign {
    pub address: String,
    pub msg: String,
    pub signature: String,
}

#[tokio::main]
async fn main() {
    let (father, url) = father_stream().await;
    println!("father: {}",father);
    let subs: Arc<Mutex<Vec<Uuid>>> = Arc::new(Mutex::new(vec![]));
    let subs_clone = subs.clone();

    let (stdin_tx, mut stdin_rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    // print all incomming;
    let (mut write, read) = ws.split();
    let handle = tokio::spawn(async move {
        read.for_each(|msg| async {
            let message: UserMessage =
                serde_json::from_str(msg.unwrap().to_string().as_str()).unwrap();
            let UserMessageData::Text(s) = message.data;
            println!("Other User Stream: {}", s);
            if message.message_type == UserMessageType::REG {
                subs_clone.lock().await.push(message.from);
            }
        })
        .await;
    });
    println!("Now you can type");
    while let Some(s) = stdin_rx.next().await {
        let lock = subs.lock().await;
        lock.iter().for_each(|id| {
            let s = s.clone();
            block_on(async {
                let user_message: UserMessage =
                    UserMessage::new_message(Uuid::from_str(&father).unwrap(), id.clone(), s);
                write
                    .send(Message::Text(serde_json::to_string(&user_message).unwrap()))
                    .await
                    .unwrap();
            });
        });
    }
    let _ = handle.await;
}

async fn father_stream() -> (String, Url) {
    let msg = String::from("123");
    let account = Account::from_mnemonic(MEMO).unwrap();
    let signature = BASE64.encode(&account.generate_sig(msg.as_bytes()).0);
    let sign = Sign {
        address: account.address().to_string(),
        msg,
        signature,
    };
    let client = reqwest::Client::new();
    let req = client
        .post("http://127.0.0.1:2001/create_stream")
        .json(&sign);
    let re = req.send().await.unwrap();
    let father = re.text().await.unwrap();
    let mut connect_addr = String::from("ws://127.0.0.1:2501/");
    connect_addr.push_str(&father);
    let url = url::Url::parse(&connect_addr).unwrap();

    return (father, url);
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
