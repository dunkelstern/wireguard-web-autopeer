use std::net::IpAddr;

use default_net::Interface;
use serde::{Deserialize, Serialize};
use serde_json;

use reqwest;

use crate::{
    storage::{IfDatabase, Peer},
    ip_net::FirstIp,
};

#[derive(Serialize, Deserialize, Debug)]
struct PeeringRequest {
    ip: IpAddr,
    prefix_len: u8,
    gateway: IpAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PeeringResponse {
    pub peers: Vec<Peer>,
}


pub fn peering_request(interface: &Interface, db: &Vec<IfDatabase>) -> PeeringResponse {
    let mut json_data: Vec<PeeringRequest> = vec![];
    
    for item in db {
        json_data.push(PeeringRequest{
            ip: item.net.addr(),
            prefix_len: item.net.prefix_len(),
            gateway: item.gateway
        });
    }
    
    let mut ips: Vec<IpAddr> = vec![];
    for v4 in interface.ipv4.iter() {
        ips.push(v4.first_ip());
    }
    for v6 in interface.ipv6.iter() {
        ips.push(v6.first_ip());
    }
    
    ips.sort();
    ips.dedup();
    
    let mut peers: Vec<Peer> = vec![];
    
    if let Ok(data) = serde_json::to_string(&json_data) {
        println!("Sending to {:?} JSON: {}", ips, data);

        for ip in ips {
            let client = reqwest::blocking::Client::new();
            let url = format!("https://{}", ip);
            //let url = format!("http://localhost:8000?{}", ip);
            let res = client.post(url)
                .json(&json_data)
                .send();

            if let Ok(response) = res {
                match response.json::<PeeringResponse>() {
                    Ok(peering_response) => {
                        for mut peer in peering_response.peers {
                            peer.interface = interface.name.clone();
                            peers.push(peer);
                        }
                    }
                    Err(error) => {
                        println!("Error decoding JSON: {:?}", error);
                    }
                }
            } else if let Err(response) = res {
                println!("ERROR: {:?}", response)
            }
        }
    }
    
    PeeringResponse { peers }
}

