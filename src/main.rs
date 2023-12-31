#![feature(noop_waker)]

#[macro_use]
extern crate lazy_static;

pub mod api_protocol;
mod data;
pub mod data_protocol;
mod handler;
mod helper;
pub mod peer_protocol;
pub mod task;

use std::{net::SocketAddr, path::PathBuf};

use dotenv::dotenv;
use futures::join;
use helper::{
    get_api_address, get_data_address, get_data_dir, get_peer_address, prepare_env, set_log,
};
use log::info;
use tokio::net::TcpListener;

use crate::{handler::{handle_data_connection, handle_peer_connection}, api_protocol::launch};
use data::init as data_init;
use task::init as task_init;

lazy_static! {
    // Directory
    static ref DATA_DIR: PathBuf = get_data_dir();
    // API address
    static ref API_ADDRESS: SocketAddr = get_api_address();
    // DATA address
    static ref DATA_ADDRESS: SocketAddr = get_data_address();
    // PEER address
    static ref PEER_ADDRESS: SocketAddr = get_peer_address();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    set_log();
    prepare_env();
    data_init().await;

    // DATA
    let try_socket = TcpListener::bind(DATA_ADDRESS.to_string()).await;
    let listener = try_socket.expect("Failed to bind");
    info!("DATA Listening on: {}", DATA_ADDRESS.to_string());

    // Let's spawn the handling of each connection in a separate task.
    let f1 = tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_data_connection(stream, addr));
        }
    });

    // PEER
    let peer_socket = TcpListener::bind(PEER_ADDRESS.to_string()).await;
    let peer_listener = peer_socket.expect("Failed to bind");
    info!("PEER Listening on: {}", PEER_ADDRESS.to_string());

    // Let's spawn the handling of each connection in a separate task.
    let f2 = tokio::spawn(async move {
        while let Ok((stream, addr)) = peer_listener.accept().await {
            tokio::spawn(handle_peer_connection(stream, addr));
        }
    });

    task_init().await;

    launch().await;

    let _ = join!(f1, f2);
    Ok(())
}
