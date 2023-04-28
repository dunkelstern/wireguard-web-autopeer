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
    pub wg_interface: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wireguard {
    pub pubkey: Option<String>,
    pub port: u16,
}

/// Network interface definition
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub net: Option<IpNet>,
    pub nexthop: Option<IpAddr>,
    pub is_default: bool,
    pub peers: Vec<Peer>,
    pub wireguard: Option<Wireguard>,
}

impl NetworkInterface {
    pub fn has_pubkey(&self) -> bool {
        if let Some(wg) = &self.wireguard {
            if let Some(_) = &wg.pubkey {
                true
            } else {
                false
            }
        } else {
            false
        }        
    }
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
    pub suspended: bool,
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
