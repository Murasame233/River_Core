// ./data/node.toml
use std::{fs, io::Read, path::PathBuf};

use crate::DATA_DIR;
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: Uuid,
    pub init_node : Option<String>,
    pub report_address: String,
}

impl Node {
    pub fn new() -> Node {
        let id = Uuid::new_v4();
        return Node { id,init_node: None ,report_address:String::new()};
    }
}

lazy_static! {
    pub static ref NODE_FILE: PathBuf = {
        let mut path = DATA_DIR.to_path_buf();
        path.push("node.toml");
        path
    };
}

pub fn init() -> Node {
    debug!("initial NODE_FILE: {}", NODE_FILE.to_str().unwrap());
    match fs::metadata(NODE_FILE.to_path_buf()) {
        Ok(_) => {
            debug!("Already initialed NODE_FILE.");
            let mut f: fs::File = std::fs::File::open(NODE_FILE.to_path_buf()).unwrap();
            let mut readed_string: String = String::new();
            let _ = f.read_to_string(&mut readed_string);
            let readed: Result<Node, toml::de::Error> = toml::from_str(&readed_string);
            if readed.is_ok() {
                return readed.unwrap();
            }
        }
        Err(_) => {
            std::fs::File::create(NODE_FILE.to_path_buf()).unwrap();
        }
    };

    let node = Node::new();

    let t = toml::to_string(&node).unwrap();
    fs::write(NODE_FILE.to_path_buf(), t).unwrap();
    debug!("NODE_FILE Initial success");
    debug!("id: {}", node.id);
    node
}
