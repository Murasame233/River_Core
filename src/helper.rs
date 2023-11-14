use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use log::info;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
};

use crate::{API_ADDRESS, DATA_DIR, PEER_ADDRESS};

pub fn set_log() {
    let stdout = ConsoleAppender::builder().build();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();
}

pub fn get_data_dir() -> PathBuf {
    let s = std::env::var("DATA_DIR");
    let mut path = std::env::current_dir().unwrap();
    if s.is_err() {
        path.push("data");
    } else {
        let subpath = PathBuf::from(&s.unwrap());
        if subpath.is_absolute() {
            return subpath;
        }
        path.push(subpath);
    }
    // if there is no this path, create it
    if !path.exists() {
        std::fs::create_dir(&path).unwrap();
    }
    return path;
}

pub fn get_api_address() -> SocketAddr {
    let s = std::env::var("API_PORT");
    let mut address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    if s.is_err() {
    } else {
        let parsed = s.unwrap().parse::<u16>();
        if parsed.is_ok() {
            address.set_port(parsed.unwrap());
        };
    }
    return address;
}

pub fn get_peer_address() -> SocketAddr {
    let s = std::env::var("PEER_PORT");
    let mut address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3001);
    if s.is_err() {
    } else {
        let parsed = s.unwrap().parse::<u16>();
        if parsed.is_ok() {
            address.set_port(parsed.unwrap());
        };
    }
    return address;
}

pub fn prepare_env() {
    info!("DATA dir: {}", DATA_DIR.to_str().unwrap());
    info!("API address: {}", API_ADDRESS.to_string());
    info!("PEER address: {}", PEER_ADDRESS.to_string());
}
