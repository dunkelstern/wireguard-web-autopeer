use std::net::{IpAddr, SocketAddr};
use if_watch::IpNet;
use serde::{Deserialize, Serialize};

/// Timeout in seconds.
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Timeout(u64);
impl Default for Timeout {
    fn default() -> Self {
        Timeout(30)
    }
}
impl From<Timeout> for u64 {
    fn from(value: Timeout) -> Self {
        value.0
    }
}

/// Peer information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Peer {
    pub pubkey: String,
    pub endpoint: Option<IpAddr>,
    pub port: Option<u16>,
    pub ip: Option<Vec<IpAddr>>,
}

/// Network interface definition
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub net: Option<IpNet>,
    pub nexthop: Option<IpAddr>,
    pub is_default: bool,
    pub peers: Vec<Peer>,
}

/// Settings
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    #[serde(default)]
    pub refresh_timeout: Timeout,
}

impl Default for Settings {
    fn default() -> Self {
        Self { refresh_timeout: Timeout::default() }
    }
}


/// Internal state
#[derive(Clone, Debug)]
pub struct StateManager {
    pub interfaces: Vec<NetworkInterface>,
    pub settings: Settings,
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
