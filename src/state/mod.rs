use if_watch::IpNet;

use crate::{network::utils::{GetInterface, next_hop}, wireguard::information::query_wg_info, http::peering::peering_request};

use self::structs::{StateManager, Settings, NetworkInterface, Peer};

pub mod structs;
pub mod messages;

impl StateManager {
    pub fn new() -> Self {
        Self{
            interfaces: vec![],
            settings: Settings::default(),
            suspended: true
        }
    }

    async fn perform_queries(&mut self) {
        // Create a peering queries
        if !self.suspended {
            for interface in self.interfaces.clone() {
                if interface.has_pubkey() {
                    match peering_request(&self, &interface).await {
                        Ok(response) => {
                            let dangling_peers = self.update_peers(response.peers, &interface);
                            // TODO: Remove dangling peers from Wireguard interfaces
                        },
                        Err(error) => error!("ERROR: {}", error),
                    }
                }
            }
        }
    }

    fn add_or_update_interface(&mut self, interface: NetworkInterface) -> bool {
        // find interface
        for mut item in &mut self.interfaces {
            if (item.name == interface.name) && (item.net == interface.net) {
                if (item.nexthop != interface.nexthop) || (item.is_default != interface.is_default) || (item.wireguard != interface.wireguard) {
                    debug!("Updating Interface: {:?}", interface);
                    item.nexthop = interface.nexthop;
                    item.is_default = interface.is_default;
                    item.wireguard = interface.wireguard.clone();
                    return true;
                }
                return false;
            }
        }

        debug!("Adding Interface: {:?}", interface);
        self.interfaces.push(interface);
        true
    }

    /// remove registered interface, removes peers of that interface if there were some
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

    /// update peers of an interface, returns peers that have been removed
    fn update_peers(&mut self, peers: Vec<Peer>, wg: &NetworkInterface) -> Vec<Peer> {
        let old_peers: Vec<Peer> = vec![];

        // find correct network interface for peers
        for interface in &mut self.interfaces {
            for peer in &peers {
                if let (Some(net), Some(endpoint)) = (interface.net, peer.endpoint) {
                    // check if the network contains the endpoint and if the do not have the peer already
                    if net.contains(&endpoint) && !interface.peers.contains(peer) {
                        interface.peers.push(peer.clone());
                        // TODO: add peers to wireguard interface
                    }
                }
            }
        }
        
        // TODO: return old peers
        old_peers
    }

    pub async fn ifup(&mut self, net: IpNet) {
        info!("Interface up event: {:?}", net);
        let interface = net.interface().unwrap();
        let netif: NetworkInterface;

        // Get next hop
        let (default, gw) = next_hop(net).await;        
        debug!("Next hop for network {:?} is {:?}", net, gw);
        netif = NetworkInterface {
            name: interface.name.clone(),
            net: Some(net),
            nexthop: gw,
            is_default: default,
            peers: vec![],
            wireguard: query_wg_info(&interface.name)
        };

        // add interface to state
        let changed = self.add_or_update_interface(netif.clone());
        if changed {
            self.perform_queries().await;
        }
    }
    
    pub async fn ifdown(&mut self, net: IpNet) {
        info!("Interface down event: {:?}", net);
        let peers = self.remove_interface(net);
        info!("Removing dangling peers: {:?}", peers);
    }
    
    pub async fn refresh(&mut self) {
        self.perform_queries().await;
    }
}
