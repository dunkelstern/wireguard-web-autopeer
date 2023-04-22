use std::{net::IpAddr, str::FromStr};
use wireguard_control::{PeerConfigBuilder, Key, DeviceUpdate, Backend, InterfaceName, Device};

use crate::storage::Peer;

pub fn add_peer(peer: &Peer, interface: &String) {
    info!("Adding peer {} to interface {}", peer.pubkey, interface);
        
    if let Ok(key) = Key::from_base64(&peer.pubkey) {        
        let mut builder = PeerConfigBuilder::new(&key)
            .replace_allowed_ips();
        
        // If we have an endpoint set the endpoint
        if let Some(socket) = peer.socket_endpoint() {
            builder = builder.set_endpoint(socket);
        }

        // If we have allowed ips, set them
        if let Some(ips) = &peer.ips {
            for ip in ips {
                let cidr = match ip {
                    IpAddr::V4(_ip) => 32,
                    IpAddr::V6(_ip) => 128
                };
            
                builder = builder.add_allowed_ip(*ip, cidr)
            }
        }
                
        if let Ok(iface) = InterfaceName::from_str(interface) {
            #[cfg(target_os = "linux")]
            let result = DeviceUpdate::new().add_peer(builder).apply(&iface, Backend::Kernel);

            #[cfg(not(target_os = "linux"))]
            let result = DeviceUpdate::new().add_peer(builder).apply(&iface, Backend::Userspace);

            if let Err(error) = result {
                error!("Error adding peer: {:?}", error);
            }
        }
    }
}

pub fn remove_peer(peer: &Peer, interface: &String) {
    info!("Removing peer {} from interface {}", peer.pubkey, interface);

    if let Ok(key) = Key::from_base64(&peer.pubkey) {
        if let Ok(iface) = InterfaceName::from_str(interface) {
            #[cfg(target_os = "linux")]
            let result = DeviceUpdate::new().remove_peer_by_key(&key).apply(&iface, Backend::Kernel);

            #[cfg(not(target_os = "linux"))]
            let result = DeviceUpdate::new().remove_peer_by_key(&key).apply(&iface, Backend::Userspace);

            if let Err(error) = result {
                error!("Error removing peer: {:?}", error);
            }
        }
    }
}

pub fn pubkey_by_interface(interface_name: InterfaceName) -> Option<String> {
    #[cfg(target_os = "linux")]
    if let Ok(device) = Device::get(&interface_name, Backend::Kernel) {
        if let Some(key) = device.public_key {
            debug!("fetching pubkey for {:?}: {:?}", interface_name, key);            
            return Some(key.to_base64())
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    if let Ok(device) = Device::get(interface_name, Backend::Userspace) {
        device.public_key
    }
    
    None
}