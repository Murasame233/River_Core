mod node;
pub mod global;
pub mod peer;

use log::info;
use node::init as node_init;

use crate::data::global::GLOBAL_NODE;

pub async fn init(){
    let node = node_init();
    info!("NODE_FILE: {}",node::NODE_FILE.to_str().unwrap());
    info!("NODE: {:?}",node);
    let mut r = GLOBAL_NODE.write().await;
    *r = node;
}