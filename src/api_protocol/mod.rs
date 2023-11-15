use ::serde::Deserialize;
use algonaut::{core::Address, crypto::Signature};
use data_encoding::BASE64;
use log::info;
use rocket::{post, routes, serde::json::Json};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    data::{
        account::Account,
        global::{StreamType, GLOBAL_USER, GLOBAL_USER_STREAM_TYPE},
    },
    API_ADDRESS, data_protocol::broad_cast_join,
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Sign {
    pub address: String,
    pub msg: String,
    pub signature: String,
}

#[post("/create_stream", data = "<sign>")]
async fn create_stream(sign: Json<Sign>) -> String {
    let s: Signature = Signature(
        BASE64
            .decode(sign.signature.as_bytes())
            .unwrap()
            .try_into()
            .unwrap(),
    );
    let address: Address = Address::from_str(&sign.address).unwrap();
    let msg = sign.msg.as_bytes();
    if address.verify_bytes(msg, &s) {
        info!("valid");
        let stream_uuid = uuid::Uuid::new_v4();
        let address_arc = Arc::new(Mutex::new(Account {
            address: address.to_string(),
            father: None,
            sub: Some(vec![]),
        }));
        GLOBAL_USER.insert(stream_uuid, address_arc).unwrap();
        let stream_type = Arc::new(Mutex::new(StreamType::Main));
        GLOBAL_USER_STREAM_TYPE
            .insert(stream_uuid, stream_type)
            .unwrap();
        info!("new main stream {}", stream_uuid.to_string());
        return stream_uuid.to_string();
    }
    return String::from("Error");
}
#[post("/join_stream", data = "<sign>")]
async fn join_stream(sign: Json<Sign>) -> String {
    let s: Signature = Signature(
        BASE64
            .decode(sign.signature.as_bytes())
            .unwrap()
            .try_into()
            .unwrap(),
    );
    let address: Address = Address::from_str(&sign.address).unwrap();
    let msg = &sign.msg;

    if address.verify_bytes(msg.as_bytes(), &s) {
        info!("valid");
        let stream_uuid = uuid::Uuid::new_v4();
        let father = Uuid::from_str(&msg).unwrap();
        let address_arc = Arc::new(Mutex::new(Account {
            address: address.to_string(),
            father: Some(father),
            sub: None,
        }));
        GLOBAL_USER.insert(stream_uuid, address_arc).unwrap();
        let stream_type = Arc::new(Mutex::new(StreamType::Sub));
        GLOBAL_USER_STREAM_TYPE
            .insert(stream_uuid, stream_type)
            .unwrap();
        tokio::spawn(broad_cast_join(stream_uuid, father));
        info!("new sub stream {}", stream_uuid.to_string());
        return stream_uuid.to_string();
    }
    return String::from("Error");
}

pub async fn launch() {
    println!("Launching API");
    let mut c = rocket::Config::default();
    c.address = API_ADDRESS.ip();
    c.port = API_ADDRESS.port();
    let _ = rocket::build()
        .configure(c)
        .mount("/", routes![create_stream, join_stream])
        .launch()
        .await;
}
