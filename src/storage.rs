use std::net::{IpAddr, SocketAddr};
use default_net::Interface;
use if_watch::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub pubkey: String,
    pub endpoint: Option<IpAddr>,
    pub port: Option<u16>,
    pub ips: Option<Vec<IpAddr>>,
    
    #[serde(default = "empty_interface")]
    pub interface: String,
}

fn empty_interface() -> String {
    "".to_string()
}

#[derive(Clone, Debug)]
pub struct IfDatabase {
    pub net: IpNet,
    pub gateway: IpAddr,
}

#[derive(Clone, Debug)]
pub struct State {
    pub if_database: Vec<IfDatabase>,
    pub peers: Vec<Peer>
}

#[derive(Clone, Debug)]
pub struct WGInterface {
    pub interface: Interface,
    pub pubkey: String,
}

pub enum Changed<T> {
    ValueChanged(T),
    ValueUnchanged(T)
}


impl Peer {
    pub fn socket_endpoint(&self) -> Option<SocketAddr> {
        if let (Some(endpoint), Some(port)) = (self.endpoint, self.port) {
            Some(SocketAddr::new(endpoint, port))
        } else {
            None
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self{if_database: vec![], peers: vec![]}
    }
}