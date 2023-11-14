use crate::peer_protocol::Message as PeerMessage;

use super::node::Node;
use super::peer::Peer;
use futures::stream::{SplitSink, SplitStream};
use scc::HashMap;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex, RwLock},
    task::JoinHandle,
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

lazy_static! {
    pub static ref GLOBAL_NODE: RwLock<Node> = RwLock::new(Node::new());

    // Peers
    pub static ref GLOBAL_PEERS: HashMap<Uuid, Arc<Mutex<Peer>>> =
        HashMap::new();

    // Streams
    pub static ref GLOBAL_SENDER: HashMap<Uuid, Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>
    > = HashMap::new();
    pub static ref GLOBAL_RECEIVER: HashMap<Uuid, Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>> =
        HashMap::new();
        pub static ref GLOBAL_HANDLE: HashMap<Uuid, Arc<Mutex<JoinHandle<()>>>> = HashMap::new();

    // Request-Reponse-Channel
    pub static ref GLOBAL_REQUEST_LOCKER: HashMap<Uuid, Sender<PeerMessage>> = HashMap::new();

    pub static ref GLOBAL_VERSION: &'static str = &"0.0.1";
}
