use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use serde_json;

use reqwest;

use crate::{
    storage::{IfDatabase, Peer, WGInterface},
    ip_net::FirstIp,
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct PeeringRequest {
    ip: IpAddr,
    #[serde(rename = "netmask")]
    prefix_len: u8,
    gateway: IpAddr,
    pubkey: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PeeringResponse {
    pub peers: Vec<Peer>,
}


pub async fn peering_request(interface: &WGInterface, db: &Vec<IfDatabase>) -> PeeringResponse {
    let mut json_data: Vec<PeeringRequest> = vec![];

    for item in db {
        json_data.push(PeeringRequest{
            ip: item.net.addr(),
            prefix_len: item.net.prefix_len(),
            gateway: item.gateway,
            pubkey: interface.pubkey.clone()
        });
    }

    let mut ips: Vec<IpAddr> = vec![];
    for v4 in &interface.interface.ipv4 {
        ips.push(v4.first_ip());
    }
    for v6 in &interface.interface.ipv6 {
        ips.push(v6.first_ip());
    }

    ips.sort();
    ips.dedup();

    let mut peers: Vec<Peer> = vec![];

    if let Ok(data) = serde_json::to_string(&json_data) {
        debug!("Sending to {:?} JSON: {}", ips, data);

        for ip in ips {
            let client = reqwest::Client::new();
            let url = match ip {
                IpAddr::V4(_ip) => format!("http://{}/peering-request", ip),
                IpAddr::V6(_ip) => format!("http://[{}]/peering-request", ip)
            };
            //let url = format!("http://localhost:8000?{}", ip);
            let res = client.post(url)
                .json(&json_data)
                .send().await;

            if let Ok(response) = res {
                debug!("Response: {:?}", response);
                match response.json::<PeeringResponse>().await {
                    Ok(peering_response) => {
                        for mut peer in peering_response.peers {
                            peer.interface = interface.interface.name.clone();
                            peers.push(peer);
                        }
                    }
                    Err(error) => {
                        error!("Error decoding JSON: {:?}", error);
                    }
                }
            } else if let Err(response) = res {
                error!("ERROR: {:?}", response)
            }
        }
    }

    PeeringResponse { peers }
}


#[cfg(test)]
mod tests {
    use super::PeeringResponse;


    #[test]
    fn deserialize_full_peer_list() {
        let mut result: PeeringResponse = serde_json::from_str(r#"{"peers": [{"pubkey": "123", "endpoint": "10.0.0.0", "port": 123, "ip": [ "10.0.0.0", "fd00::1"]}]}"#).unwrap();
        assert_eq!(result.peers.len(), 1);
        let peer = result.peers.pop().unwrap();
        assert_eq!(peer.pubkey, "123");
        assert_eq!(peer.endpoint, Some("10.0.0.0".parse().unwrap()));
        assert_eq!(peer.port, Some(123));
        let ips = peer.ips.unwrap();
        assert_eq!(ips.len(), 2);
    }

    #[test]
    fn deserialize_pubkey_peer_list() {
        let mut result: PeeringResponse = serde_json::from_str(r#"{"peers": [{"pubkey": "123"}]}"#).unwrap();
        assert_eq!(result.peers.len(), 1);
        let peer = result.peers.pop().unwrap();
        assert_eq!(peer.pubkey, "123");
    }
}