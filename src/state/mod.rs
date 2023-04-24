use if_watch::IpNet;

use crate::network::utils::{GetInterface, next_hop};

use self::structs::{StateManager, Settings, NetworkInterface, Peer};

pub mod structs;
pub mod messages;

impl StateManager {
    pub fn new() -> Self {
        Self{
            interfaces: vec![],
            settings: Settings::default(),
        }
    }

    fn add_or_update_interface(&mut self, interface: NetworkInterface) {
        // find interface
        let mut found = false;
        for mut item in &mut self.interfaces {
            if (item.name == interface.name) && (item.net == interface.net) {
                item.nexthop = interface.nexthop;
                item.is_default = interface.is_default;
                found = true;
                break;
            }
        }

        if !found {
            self.interfaces.push(interface);
        }
    }

    fn remove_interface(&mut self, net: IpNet) -> Vec<Peer> {        
        let result = self.interfaces
            .clone()
            .into_iter()
            .filter(|item| (item.net == Some(net)))
            .flat_map(|item| item.peers)
            .collect();
        
        self.interfaces.retain(|item| (item.net != Some(net)));
        
        result
    }

    pub async fn ifup(&mut self, net: IpNet) {
        info!("Interface up event: {:?}", net);

        // Get next hop
        if let Some((default, nexthop)) = next_hop(net).await {
            debug!("Next hop for network {:?} is {:?}", net, nexthop.gateway.unwrap());
            // add interface to state
            let interface = net.interface().unwrap();
            self.add_or_update_interface(
                NetworkInterface {
                    name: interface.name,
                    net: Some(net),
                    nexthop: nexthop.gateway,
                    is_default: default,
                    peers: vec![]
                }
            )
            
        } else {
            debug!("No next hop for network {:?}", net);
        }
    }
    
    pub async fn ifdown(&mut self, net: IpNet) {
        info!("Interface down event: {:?}", net);
        let peers = self.remove_interface(net);
        info!("Removing dangling peers: {:?}", peers);
    }
    
    pub async fn refresh(&self) {
        //todo!();
    }
}

// pub async fn update(mut state: State) -> State {
//     let wg_interfaces = wireguard_interfaces();

//     for interface in wg_interfaces {
//         let response = peering_request(&interface, &state.if_database).await;

//         // remove peers
//         for peer in &state.peers {
//             if interface.interface.name == peer.interface {
//                 remove_peer(peer, &interface.interface.name);
//             }
//         }

//         // add new peers and dedup internal state
//         state.peers.extend(response.peers.clone());
//         state.peers.sort_by(|a, b| a.pubkey.cmp(&b.pubkey));
//         state.peers.dedup_by(|a, b| a.pubkey == b.pubkey && a.ips == b.ips && a.interface == b.interface);
        
//         // add peers to wireguard
//         for peer in &response.peers {
//             add_peer(peer, &interface.interface.name);
//         }
//     }

//     state
// }