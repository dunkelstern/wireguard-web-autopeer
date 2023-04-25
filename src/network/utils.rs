use if_watch::IpNet;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use net_route::Handle;


pub trait GetInterface {
    fn interface_name(&self) -> Option<String>;
    fn interface(&self) -> Option<NetworkInterface>;
}

impl GetInterface for IpNet {
    fn interface_name(&self) -> Option<String> {
        if let Some(interface) = self.interface(){
            return Some(interface.name);
        }
        None
    }

    fn interface(&self) -> Option<NetworkInterface> {
        let interfaces = NetworkInterface::show().unwrap();
        for interface in &interfaces {
            for addr in &interface.addr {
                if self.contains(&addr.ip()) {
                    return Some(interface.clone());
                }
            }
        }
    
        debug!("No interface for {:?}?", self);
        None
    }
}


pub trait FirstIp {
    fn first_ip(&self) -> IpAddr;
}

impl FirstIp for IpNet {
    fn first_ip(&self) -> IpAddr {
        match self {
            IpNet::V4(net) => {
                if net.prefix_len() == 32 {
                    IpAddr::V4(net.addr())
                } else {
                    let mask = u32::MAX << (32 - net.prefix_len());
                    let mut ip: u32 = net.addr().into();
                    ip = (ip & mask) + 1;
                    IpAddr::V4(Ipv4Addr::from(ip))
                }
            }
            IpNet::V6(net) => {
                if net.prefix_len() == 128 {
                    IpAddr::V6(net.addr())
                } else {
                    let mask = u128::MAX << (128 - net.prefix_len());
                    let mut ip: u128 = net.addr().into();
                    ip = (ip & mask) + 1;
                    IpAddr::V6(Ipv6Addr::from(ip))
                }
            }
        }        
    }    
}

pub async fn next_hop(net: IpNet) -> (bool, Option<IpAddr>) {
    match Handle::new() {
        Ok(handle) => {
            if let Ok(routes) = handle.list().await {
                // Find next hop
                for route in &routes {
                    if (net.prefix_len() == route.prefix) && (net.contains(&route.destination)) {
                        if let Some(_) = route.gateway {
                            return (false, route.gateway.clone());
                        }
                    }
                }
                // No next hop, find standard gateway
                if let Some(interface) = net.interface() {
                    for route in &routes {
                        if (route.prefix != 0) || (route.ifindex.unwrap() != interface.index) {
                            continue;
                        }
                        if let Some(gateway) = route.gateway {
                            if let (IpNet::V4(_), IpAddr::V4(_)) = (net, gateway) {
                                return (true, route.gateway.clone());
                            }
                            if let (IpNet::V6(_), IpAddr::V6(_)) = (net, gateway) {
                                return (true, route.gateway.clone());
                            }
                        }
                    }                        
                }
            }
        }
        Err(error) => {
            error!("Could not fetch routing table: {:?}", error);
        }
    }
    
    (false, None)
}


#[cfg(test)]
mod tests {
    use if_watch::IpNet;
    use std::{net::IpAddr, str::FromStr};

    use crate::network::utils::FirstIp;
   
    #[test]
    fn first_ip_in_net_v4() {
        assert_eq!(IpNet::from_str("10.0.0.10/8").unwrap().first_ip(), "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(IpNet::from_str("10.1.0.10/8").unwrap().first_ip(), "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(IpNet::from_str("192.168.1.10/24").unwrap().first_ip(), "192.168.1.1".parse::<IpAddr>().unwrap());
        assert_eq!(IpNet::from_str("192.168.1.10/32").unwrap().first_ip(), "192.168.1.10".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn first_ip_in_net_v6() {
        assert_eq!(IpNet::from_str("fd12:3456:789a:1::/64").unwrap().first_ip(), "fd12:3456:789a:1::1".parse::<IpAddr>().unwrap());
        assert_eq!(IpNet::from_str("fd12:3456:789a:1::/8").unwrap().first_ip(), "fd00::1".parse::<IpAddr>().unwrap());
    }

}