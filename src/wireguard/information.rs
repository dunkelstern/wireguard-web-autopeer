use std::net::SocketAddr;

use wireguard_uapi::{DeviceInterface, WgSocket, set::WgPeerF};
use base64::{Engine as _, engine::general_purpose};

use wireguard_uapi::set::{AllowedIp, Device, Peer};


use crate::state::structs::{Wireguard, self};

pub fn query_wg_info(device_name: &String) -> Option<Wireguard> {
    if let Ok(mut wg) = WgSocket::connect() {
        if let Ok(device) = wg.get_device(DeviceInterface::from_name(device_name)) {
            if let Some(pubkey) = device.public_key {
                return Some(Wireguard {
                    pubkey: Some(general_purpose::STANDARD.encode(pubkey)),
                    port: device.listen_port
                })
            } else {
                return Some(Wireguard{pubkey: None, port: device.listen_port});
            }
        }
    }
    None
}

pub fn add_peer<'a>(peer: structs::Peer) -> Result<(), wireguard_uapi::err::SetDeviceError> {
    if let Ok(mut wg) = WgSocket::connect() {
        let wg_interface = peer.wg_interface.unwrap();
        let interface = DeviceInterface::from_name(wg_interface);
        let key = general_purpose::STANDARD.decode(peer.pubkey).unwrap();

        // convert values
        let k: [u8; 32] = key.try_into().unwrap();
        let mut p = Peer{
            public_key: &k,
            flags: vec![],
            preshared_key: None,
            endpoint: None,
            persistent_keepalive_interval: None,
            allowed_ips: vec![],
            protocol_version: None
        };
        
        // If we have an endpoit set it
        let socket: SocketAddr;
        if let (Some(endpoint), Some(port)) = (peer.endpoint, peer.port) {
            socket = SocketAddr::new(endpoint, port);
            p.endpoint = Some(&socket);
        }
        
        // If we have allowed ips, set them
        if let Some(ips) = &peer.ip {
            let mut allowed_ips: Vec<AllowedIp> = vec![];
            for ip in ips {
                allowed_ips.push(AllowedIp::from_ipaddr(ip));
            }
            p.allowed_ips = allowed_ips;
        }
        
        // create a query
        let dev = Device {
            interface,
            flags: vec![],
            private_key: None,
            listen_port: None,
            fwmark: None,
            peers: vec![p],
        };

        return wg.set_device(dev);
    }
    return Ok(());
}

pub fn remove_peer(peer: structs::Peer) -> Result<(), wireguard_uapi::err::SetDeviceError> {
    if let Ok(mut wg) = WgSocket::connect() {
        let wg_interface = peer.wg_interface.unwrap();
        let interface = DeviceInterface::from_name(wg_interface);
        let key = general_purpose::STANDARD.decode(peer.pubkey).unwrap();

        // convert values
        let k: [u8; 32] = key.try_into().unwrap();
        let mut p = Peer{
            public_key: &k,
            flags: vec![WgPeerF::RemoveMe],
            preshared_key: None,
            endpoint: None,
            persistent_keepalive_interval: None,
            allowed_ips: vec![],
            protocol_version: None
        };
        
        // If we have an endpoit set it
        let socket: SocketAddr;
        if let (Some(endpoint), Some(port)) = (peer.endpoint, peer.port) {
            socket = SocketAddr::new(endpoint, port);
            p.endpoint = Some(&socket);
        }
        
        // If we have allowed ips, set them
        if let Some(ips) = &peer.ip {
            let mut allowed_ips: Vec<AllowedIp> = vec![];
            for ip in ips {
                allowed_ips.push(AllowedIp::from_ipaddr(ip));
            }
            p.allowed_ips = allowed_ips;
        }
        
        // create a query
        let dev = Device {
            interface,
            flags: vec![],
            private_key: None,
            listen_port: None,
            fwmark: None,
            peers: vec![p],
        };

        return wg.set_device(dev);
    }
    return Ok(());
}