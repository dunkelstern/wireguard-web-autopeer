#[macro_use]
extern crate log;

use http::peering_request;
use monitor::monitor;
use network_interface::wireguard_interfaces;
use storage::State;
use wireguard::{add_peer, remove_peer};
use tray_item::TrayItem;
use std::thread;

#[cfg(target_os = "linux")]
use gtk;


#[cfg(target_os = "windows")]
use {std::sync::mpsc, tray_item::TrayItem};

enum Message {
    Quit,
    Refresh
}

pub mod storage;
pub mod ip_net;
mod network_interface;
mod http;
mod monitor;
mod wireguard;


pub fn update(mut state: State) -> State {
    let wg_interfaces = wireguard_interfaces();

    for interface in wg_interfaces {
        let response = peering_request(&interface, &state.if_database);

        // remove peers
        for peer in &state.peers {
            if interface.interface.name == peer.interface {
                remove_peer(peer, &interface.interface.name);
            }
        }

        // add new peers and dedup internal state
        state.peers.extend(response.peers.clone());
        state.peers.sort_by(|a, b| a.pubkey.cmp(&b.pubkey));
        state.peers.dedup_by(|a, b| a.pubkey == b.pubkey && a.ips == b.ips && a.interface == b.interface);
        
        // add peers to wireguard
        for peer in &response.peers {
            add_peer(peer, &interface.interface.name);
        }
    }

    state
}

fn main() {
    env_logger::init();

    #[cfg(target_os = "linux")]
    if let Ok(_gtk) = gtk::init() {

        // FIXME: Remove all peers on shutdown
        thread::spawn(move || {
            monitor(update);
        });

        let mut tray = TrayItem::new("Wireguard-Web-Autopeer", "wireguard-web-autopeer").unwrap();

        tray.add_label("Wireguard-Web").unwrap();

        // FIXME: Add a refresh menu item
        //
        // tray.add_menu_item("Refresh Peers", || {
        // }).unwrap();


        #[cfg(target_os = "linux")]
        {
            tray.add_menu_item("Quit", || {
                gtk::main_quit();
            }).unwrap();
            gtk::main();
        }

        // FIXME: this is wrong
        #[cfg(target_os = "macos")]
        {
            let mut inner = tray.inner_mut();
            inner.add_quit_item("Quit");
            inner.display();
        }

        // FIXME: this is wrong
        #[cfg(target_os = "windows")]
        {
            let (tx, rx) = mpsc::channel();
            tray.add_menu_item("Quit", move || {
                tx.send(Message::Quit).unwrap();
            }).unwrap();

            loop {
                match rx.recv() {
                    Ok(Message::Quit) => break,
                    _ => {}
                }
            }
        }
    } else {
        // No systray, probably running headless or as root
        monitor(update);
    }
}
