#[macro_use]

extern crate log;

// Crate modules
mod state;
mod tray;
mod autorefresh;
mod network;
mod wireguard;
mod http;

// Everything tokio
use tokio::{select, sync::mpsc::channel, signal::ctrl_c};
use tokio_util::sync::CancellationToken;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(windows)]
use tokio::signal::windows::{ctrl_break, ctrl_close};

// Systray
use tray::Tray;

// State keeping
use state::messages::Message;
use state::structs::StateManager;

// Services
use autorefresh::autorefresh;
use network::monitor::monitor;


// Main loop
#[tokio::main]
async fn main() {
    // logging
    env_logger::init();
        
    // local state
    let mut state = StateManager::new();
    let (eventbus_tx, mut eventbus_rx) = channel::<Message>(32);
    info!("Running with settings: {}", serde_json::to_string(&state.settings).unwrap());

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

    // On windows we use CTRL+Break for reload and Console close for term
    #[cfg(windows)]
    let mut hup = ctrl_break().unwrap();
    #[cfg(windows)]
    let mut term = ctrl_close().unwrap();

    // cancellation tokens to suspend background tasks
    let mut background_tasks = CancellationToken::new();

    // start auto refresh loop
    let mut refresh_handle = Some(autorefresh(eventbus_tx.clone(), background_tasks.clone(), state.settings.refresh_timeout));    
    let mut monitor_handle = Some(monitor(eventbus_tx.clone(), background_tasks.clone()));

    debug!("Entering main event loop...");
    'main: loop {
        select! {
            // receive a message
            message = eventbus_rx.recv() => {
                match message.unwrap() {
                    Message::Quit => {
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
                        if let Some(handle) = monitor_handle {
                            let _ = &handle.await.unwrap();
                            monitor_handle = None;
                        }
                        state.suspended = true;
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
                            None => Some(autorefresh(eventbus_tx.clone(), background_tasks.clone(), state.settings.refresh_timeout))
                        };
                        monitor_handle = match monitor_handle {
                            Some(handle) => Some(handle),
                            None => Some(monitor(eventbus_tx.clone(), background_tasks.clone()))
                        };
                    }
                    Message::InterfacesLoaded => {
                        state.suspended = false;
                        state.refresh().await;
                    }
                    Message::InterfaceUp(interface) => state.ifup(interface).await,
                    Message::InterfaceDown(interface) => state.ifdown(interface).await,
                    Message::RefreshPeers => state.refresh().await,
                }
            }
            _ = hup.recv() => {
                info!("Received HUP, Reloading all peers...");
                state.refresh().await;
            }
            _ = term.recv() => {
                info!("Received TERM, Shutting down...");
                break 'main;
            }
            _ = ctrl_c() => {
                info!("Received CTRL+C, Shutting down...");
                break 'main;
            }
        }
    }
    
    // Shutdown all services
    if let Some(tx) = &tray_tx {
        tx.send(Message::Quit).await.expect("Failed to send quit messaage to systray");
    }
    background_tasks.cancel();
    if let Some(handle) = refresh_handle {
        let _ = &handle.await.unwrap();
    }
    if let Some(handle) = monitor_handle {
        let _ = &handle.await.unwrap();
    }

}
