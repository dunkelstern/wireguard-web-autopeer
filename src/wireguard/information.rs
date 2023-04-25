use wireguard_uapi::{DeviceInterface, WgSocket};
use base64::{Engine as _, engine::general_purpose};

use crate::state::structs::Wireguard;

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
