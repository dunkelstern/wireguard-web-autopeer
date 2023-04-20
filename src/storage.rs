use std::net::{IpAddr, SocketAddr};
use if_watch::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub pubkey: String,
    pub endpoint: IpAddr,
    pub port: u16,
    pub ip: IpAddr,
    
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

pub enum Changed<T> {
    ValueChanged(T),
    ValueUnchanged(T)
}


impl Peer {
    pub fn socket_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.endpoint, self.port)
    }
}

impl State {
    pub fn new() -> Self {
        Self{if_database: vec![], peers: vec![]}
    }
}