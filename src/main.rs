#[macro_use]
extern crate log;

use state::messages::Message;
use tokio::{sync::mpsc::{channel, Sender}, task::JoinHandle};

use tokio::signal::ctrl_c;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(windows)]
use tokio::signal::windows::{ctrl_break, ctrl_close};

use tokio_util::sync::CancellationToken;
use tokio::select;
use tray::Tray;

use crate::state::structs::StateManager;

mod state;
mod tray;


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

fn autorefresh(tx: Sender<Message>, cancel: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            select! {
                // cancelled, break loop, exit task
                _ = cancel.cancelled() => {
                    break;
                }
                // 60 seconds timeout, send refresh message
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                    let _ = tx.send(Message::RefreshPeers).await.unwrap();
                }
            }
        }
    })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // local state
    let state = StateManager::new();
    let (eventbus_tx, mut eventbus_rx) = channel::<Message>(32);

    // Systray
    let (tray, tray_tx) = Tray::try_new(eventbus_tx.clone());
    if let Some(mut tray) = tray {
        tokio::spawn(async move {
            tray.run().await;
        });
    }

    // HUP and Term signals
    #[cfg(unix)]
    let mut hup = signal(SignalKind::hangup()).unwrap();
    #[cfg(unix)]
    let mut term = signal(SignalKind::terminate()).unwrap();

    #[cfg(windows)]
    let mut hup = ctrl_break().unwrap();
    #[cfg(windows)]
    let mut term = ctrl_close().unwrap();

    // cancellation tokens to suspend background tasks
    let mut background_tasks = CancellationToken::new();

    // start auto refresh loop
    let mut refresh_handle = Some(autorefresh(eventbus_tx.clone(), background_tasks.clone()));    

    debug!("Entering main event loop...");
    'main: loop {
        select! {
            // receive a message
            message = eventbus_rx.recv() => {
                debug!("Received Message: {:?}", message);
                match message.unwrap() {
                    Message::Quit => {
                        if let Some(tx) = &tray_tx {
                            tx.send(Message::Quit).await.expect("Failed to send quit messaage to systray");
                        }
                        // Suspend Network monitor and Automatic refresh
                        background_tasks.cancel();
                        if let Some(handle) = refresh_handle {
                            let _ = &handle.await.unwrap();
                        }
                        break 'main;
                    }
                    Message::Suspend => {
                        if let Some(tx) = &tray_tx {
                            tx.send(Message::Suspend).await.expect("Failed to send suspend messaage to systray");
                        }
                        // Suspend Network monitor and Automatic refresh
                        background_tasks.cancel();
                        if let Some(handle) = refresh_handle {
                            let _ = &handle.await.unwrap();
                            refresh_handle = None;
                        }
                    }
                    Message::Resume => {
                        if let Some(tx) = &tray_tx {
                            tx.send(Message::Resume).await.expect("Failed to send resume messaage to systray");
                        }
                        background_tasks = CancellationToken::new();
                        // Start Network monitor and Automatic refresh
                        // network_monitor(eventbus_tx.clone(), backgroundTasks.clone());
                        refresh_handle = match refresh_handle {
                            Some(handle) => Some(handle),
                            None => Some(autorefresh(eventbus_tx.clone(), background_tasks.clone()))
                        }
                    }
                    Message::InterfaceUp(interface) => state.ifup(&interface),
                    Message::InterfaceDown(interface) => state.ifdown(&interface),
                    Message::RefreshPeers => {
                        // Iterate over all interfaces and call ifup on StateManager
                        for interface in &state.interfaces {
                            state.ifup(&interface.name);
                        }
                    }
                }
            }
            _ = hup.recv() => {
                info!("Received HUP, Reloading all peers...");
                // Iterate over all interfaces and call ifup on StateManager
                for interface in &state.interfaces {
                    state.ifup(&interface.name);
                }                
            }
            _ = term.recv() => {
                info!("Received TERM, Shutting down...");
                if let Some(tx) = &tray_tx {
                    tx.send(Message::Quit).await.expect("Failed to send quit messaage to systray");
                }
                // Suspend Network monitor and Automatic refresh
                background_tasks.cancel();
                if let Some(handle) = refresh_handle {
                    let _ = &handle.await.unwrap();
                }
                break 'main;
            }
            _ = ctrl_c() => {
                info!("Received CTRL+C, Shutting down...");
                if let Some(tx) = &tray_tx {
                    tx.send(Message::Quit).await.expect("Failed to send quit messaage to systray");
                }
                // Suspend Network monitor and Automatic refresh
                background_tasks.cancel();
                if let Some(handle) = refresh_handle {
                    let _ = &handle.await.unwrap();
                }
                break 'main;
            }
        }
    }
}
