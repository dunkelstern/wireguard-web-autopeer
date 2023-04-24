use std::net::{IpAddr, SocketAddr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Peer {
    pub pubkey: String,
    pub endpoint: Option<IpAddr>,
    pub port: Option<u16>,
    pub ip: Option<Vec<IpAddr>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: Option<IpAddr>,
    pub nexthop: Option<IpAddr>,
    pub prefix_len: u8,
    pub is_default: bool,
    pub peers: Vec<Peer>,
}

#[derive(Clone, Debug)]
pub struct StateManager {
    pub interfaces: Vec<NetworkInterface>,
}

impl TryFrom<Peer> for SocketAddr {
    type Error = ();

    fn try_from(value: Peer) -> Result<Self, Self::Error> {
        if let (Some(endpoint), Some(port)) = (value.endpoint, value.port) {
            Ok(SocketAddr::new(endpoint, port))
        } else {
            Err(())
        }
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self{interfaces: vec![]}
    }
    
    pub fn ifup(&self, interface: &String) {
        info!("Interface up event: {:?}", interface);

    }
    
    pub fn ifdown(&self, interface: &String) {
        info!("Interface down event: {:?}", interface);

    }
}