use default_net::{self, Interface};
use if_watch::IpNet;

#[cfg(target_os = "linux")]
use wireguard_control::backends::kernel::enumerate as enumerate_wireguard;

#[cfg(not(target_os = "linux"))]
use wireguard_control::backends::userspace::enumerate as enumerate_wireguard;

use crate::{storage::{IfDatabase, Changed, WGInterface}, wireguard::pubkey_by_interface};


pub fn if_up(net: IpNet, interfaces: Vec<Interface>, db: &Vec<IfDatabase>) -> Changed<Vec<IfDatabase>> {
    // get gateway for this interface
    // - if there is none, this is not interesting
    // - if there is one, add this interface to our watchlist
    let mut changed = false;
    let mut result = db.clone();

    for interface in interfaces {
        if let Some(gateway) = interface.gateway {
            if net.contains(&gateway.ip_addr) {
                info!("Net up: {:?}, Gateway IP: {}", net, gateway.ip_addr);
                changed = true;
                result.push(IfDatabase{net, gateway:gateway.ip_addr});
            }
        }
    }

    if changed {
        Changed::ValueChanged(result.to_vec())
    } else {
        Changed::ValueUnchanged(result.to_vec())
    }
}


pub fn if_down(net: IpNet, _interfaces: Vec<Interface>, db: &Vec<IfDatabase>) -> Changed<Vec<IfDatabase>> {
    // if the interface is on our watchlist remove it
    let mut changed = false;
    let mut result = db.clone();

    for item in db.iter() {
        if item.net == net {
            changed = true
        }
    }
    result.retain(|item| item.net != net);

    if changed {
        Changed::ValueChanged(result.to_vec())
    } else {
        Changed::ValueUnchanged(result.to_vec())
    }
}

pub fn wireguard_interfaces() -> Vec<WGInterface> {
    let mut result: Vec<WGInterface> = vec![];
    let interfaces = default_net::get_interfaces();

    if let Ok(wg_interfaces) = enumerate_wireguard() {
        for interface in wg_interfaces {

            for netif in interfaces.iter() {
                if (netif.ipv4.len() == 0) && (netif.ipv6.len() == 0) {
                    continue;
                }
                if netif.name == interface.to_string() {
                    if let Some(pubkey) = pubkey_by_interface(interface) {
                        result.push(
                            WGInterface{
                                interface: netif.clone(),
                                pubkey
                            }
                        );
                    }
                }
            }
        }
    }

    result
}


#[cfg(test)]
mod tests {
    use default_net::{Interface, Gateway};
    use default_net::interface::{InterfaceType, MacAddr};
    use default_net::ip::Ipv4Net;

    use crate::network_interface::{IfDatabase, if_up, if_down, Changed};

    fn interfaces() -> Vec<Interface> {
        vec![
            Interface{
                index: 0,
                name: "eth0".to_string(),
                friendly_name: None,
                description: None,
                if_type: InterfaceType::Ethernet,
                mac_addr: None,
                ipv4: vec![Ipv4Net{addr: "10.0.0.10".parse().unwrap(), prefix_len: 8, netmask:"255.0.0.0".parse().unwrap()}],
                ipv6: vec![],
                flags: 0,
                transmit_speed: Some(100000000),
                receive_speed: Some(100000000),
                gateway: None
            },
            Interface{
                index: 1,
                name: "eth1".to_string(),
                friendly_name: None,
                description: None,
                if_type: InterfaceType::Ethernet,
                mac_addr: None,
                ipv4: vec![Ipv4Net{addr: "192.168.1.10".parse().unwrap(), prefix_len: 24, netmask:"255.255.255.0".parse().unwrap()}],
                ipv6: vec![],
                flags: 0,
                transmit_speed: Some(100000000),
                receive_speed: Some(100000000),
                gateway: Some(Gateway{
                    mac_addr: MacAddr::zero(),
                    ip_addr: "192.168.1.1".parse().unwrap()
                })
            }
        ]
    }

    #[test]
    fn add_remove_interface() {
        let mut db: Vec<IfDatabase> = vec![];
        db = match if_up("192.168.1.10/24".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => db,
            Changed::ValueUnchanged(db) => {
                assert!(false);
                db
            }
        };
        assert_eq!(db.len(), 1);
        db = match if_down("192.168.1.10/24".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => db,
            Changed::ValueUnchanged(db) => {
                assert!(false);
                db
            }
        };
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn mixed_add_interface() {
        let mut db: Vec<IfDatabase> = vec![];
        db = match if_up("192.168.1.10/24".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => db,
            Changed::ValueUnchanged(db) => {
                assert!(false);
                db
            }
        };
        db = match if_up("10.0.0.10/8".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => {
                assert!(false);
                db
            }
            Changed::ValueUnchanged(db) => db
        };
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn no_remove_interface() {
        let mut db: Vec<IfDatabase> = vec![];

        db = match if_up("192.168.1.10/24".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => db,
            Changed::ValueUnchanged(db) => {
                assert!(false);
                db
            }
        };
        db = match if_up("10.0.0.10/8".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => {
                assert!(false);
                db
            }
            Changed::ValueUnchanged(db) => db
        };
        db = match if_down("10.0.0.10/8".parse().unwrap(), interfaces(), &mut db) {
            Changed::ValueChanged(db) => {
                assert!(false);
                db
            }
            Changed::ValueUnchanged(db) => db
        };
        assert_eq!(db.len(), 1);
    }
}