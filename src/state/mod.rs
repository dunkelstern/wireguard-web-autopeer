use if_watch::IpNet;

use crate::{network::utils::{GetInterface, next_hop}, wireguard::information::{query_wg_info, add_peer, remove_peer}, http::peering::peering_request};

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
                    info!("Performing peering query on interface {} for {}...", interface.name, interface.net.unwrap());
                    match peering_request(&self, &interface).await {
                        Ok(response) => self.update_peers(response.peers, &interface),
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

    /// update peers of an interface, returns peers that have been removed
    fn update_peers(&mut self, peers: Vec<Peer>, wg: &NetworkInterface) {
        let mut old_peers: Vec<Peer> = vec![];

        for interface in &self.interfaces {
            old_peers.extend(interface.peers.clone());
        }

        // find correct network interface for peers
        for peer in &peers {
            for interface in &mut self.interfaces {
                if let (Some(net), Some(endpoint)) = (interface.net, peer.endpoint) {
                    // check if the network contains the endpoint and if the do not have the peer already
                    if net.contains(&endpoint) {
                        if !interface.peers.contains(peer) {
                            interface.peers.push(peer.clone());
                            // add peers to wireguard interface
                            match add_peer(peer.clone()) {
                                Ok(_) => info!("Added peer {:?} @ {} to interface {:?}", endpoint, wg.name, net),
                                Err(error) => error!("Error adding peer {:?} @ {} to interface {:?}: {:?}", endpoint, wg.name, net, error),
                            }
                        }

                        // only add peer to first interface
                        break;
                    }
                }
            }
        }

        // clean out old peer list
        old_peers.retain(|peer| {
            if let Some(interface_name) = &peer.wg_interface {
                if interface_name != &wg.name {
                    return false;
                }
            }
            return !peers.contains(peer);
        });

        for interface in &mut self.interfaces {
            interface.peers.retain(|item| !old_peers.contains(item));
        }

        // Remove old peers from wireguard interfaces
        for peer in old_peers {
            if let Some(endpoint) = peer.endpoint {
                match remove_peer(peer) {
                    Ok(_) => info!("Removed peer {:?} @ {}", endpoint, wg.name),
                    Err(error) => error!("Error removing peer {:?} @ {}: {:?}", endpoint, wg.name, error),
                }
            } else {
                match remove_peer(peer.clone()) { // why the clone?
                    Ok(_) => info!("Removed peer {:?} @ {}", &peer.pubkey, wg.name),
                    Err(error) => error!("Error removing peer {:?} @ {}: {:?}", &peer.pubkey, wg.name, error),
                }
            }
        }
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
        let result: Vec<Peer> = self.interfaces
            .clone()
            .into_iter()
            .filter(|item| (item.net == Some(net)))
            .flat_map(|item| item.peers)
            .collect();

        self.interfaces.retain(|item| (item.net != Some(net)));
        
        // Remove peers from wireguard
        for peer in result {
            match remove_peer(peer.clone()) { // why the clone?
                Ok(_) => info!("Removed peer {:?} @ {}", &peer.pubkey, &peer.wg_interface.unwrap()),
                Err(error) => error!("Error removing peer {:?} @ {}: {:?}", &peer.pubkey, &peer.wg_interface.unwrap(), error),
            }
        }
    }
    
    pub async fn refresh(&mut self) {
        self.perform_queries().await;
    }
}
