use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use serde_json;

use reqwest;

use crate::state::structs::{Peer, StateManager, NetworkInterface};
use crate::network::utils::FirstIp;

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


pub async fn peering_request(state: &StateManager, wg: &NetworkInterface) -> Result<PeeringResponse, String> {
    let mut json_data: Vec<PeeringRequest> = vec![];

    for item in &state.interfaces {
        if !item.is_default {
            continue;
        }
        if let (Some(gw), Some(net)) = (item.nexthop, item.net) {
            json_data.push(PeeringRequest{
                ip: net.addr(),
                prefix_len: net.prefix_len(),
                gateway: gw,
                pubkey: wg.wireguard.clone().unwrap().pubkey.unwrap()
            });
        }
    }

    let ip = wg.net.unwrap().first_ip();

    if let Ok(data) = serde_json::to_string(&json_data) {
        debug!("Sending to {:?} JSON: {}", ip, data);

        let client = reqwest::Client::new();
        let url = match ip {
            IpAddr::V4(_ip) => format!("http://{}/peering-request", ip),
            IpAddr::V6(_ip) => format!("http://[{}]/peering-request", ip)
        };
        let res = client.post(url)
            .json(&json_data)
            .send().await;
        match res {
            Ok(response) => {
                match response.text().await {
                    Ok(result) => {
                        match serde_json::from_str::<PeeringResponse>(&result) {
                            Ok(data) => return Ok(data),
                            Err(error) => return Err(format!("{}: {}", error.to_string(), &result)),
                        }
                    }
                    Err(error) => return Err(error.to_string()),
                }
            }
            Err(error) => {
                return Err(error.to_string());
            }
        }
    }

    Err("Could not serialize data".into())
}
