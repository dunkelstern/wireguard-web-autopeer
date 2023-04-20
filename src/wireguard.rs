use std::{net::IpAddr, str::FromStr};
use wireguard_control::{PeerConfigBuilder, Key, DeviceUpdate, Backend, InterfaceName};

use crate::storage::Peer;

pub fn add_peer(peer: &Peer, interface: &String) {
    println!("Adding peer {} to interface {}", peer.pubkey, interface);
    
    let socket = peer.socket_endpoint();
    let cidr = match peer.ip {
        IpAddr::V4(_ip) => 32,
        IpAddr::V6(_ip) => 128
    };
    
    if let Ok(key) = Key::from_base64(&peer.pubkey) {        
        let builder = PeerConfigBuilder::new(&key)
            .replace_allowed_ips()
            .add_allowed_ip(peer.ip, cidr)
            .set_endpoint(socket);
        
        if let Ok(iface) = InterfaceName::from_str(interface) {
            #[cfg(target_os = "linux")]
            let result = DeviceUpdate::new().add_peer(builder).apply(&iface, Backend::Kernel);
            
            #[cfg(not(target_os = "linux"))]
            let result = DeviceUpdate::new().add_peer(builder).apply(&iface, Backend::Userspace);
            
            if let Err(error) = result {
                println!("Error adding peer: {:?}", error);
            } 
        }
    }
}

pub fn remove_peer(peer: &Peer, interface: &String) {
    println!("Removing peer {} from interface {}", peer.pubkey, interface);
        
    if let Ok(key) = Key::from_base64(&peer.pubkey) {                
        if let Ok(iface) = InterfaceName::from_str(interface) {
            #[cfg(target_os = "linux")]
            let result = DeviceUpdate::new().remove_peer_by_key(&key).apply(&iface, Backend::Kernel);
            
            #[cfg(not(target_os = "linux"))]
            let result = DeviceUpdate::new().remove_peer_by_key(&key).apply(&iface, Backend::Userspace);
            
            if let Err(error) = result {
                println!("Error removing peer: {:?}", error);
            } 
        }
    }
}