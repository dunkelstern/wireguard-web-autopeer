use futures::StreamExt;
use if_watch::{tokio::IfWatcher, IfEvent};
use tokio::{sync::mpsc::Sender, task::JoinHandle, select};
use tokio_util::sync::CancellationToken;

use crate::state::{messages::Message};


pub fn monitor(tx: Sender<Message>, cancel: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut watcher = IfWatcher::new().unwrap();
        loop {
            select! {
                // cancelled, break loop, exit task
                _ = cancel.cancelled() => {
                    break;
                }
                // New network interface event
                event = watcher.select_next_some() => {
                    match event {
                        Ok(IfEvent::Up(ip)) => {
                            if !ip.addr().is_loopback() {
                                tx.send(Message::InterfaceUp(ip)).await.unwrap();
                            }
                        }
                        Ok(IfEvent::Down(ip)) => {
                            if !ip.addr().is_loopback() {
                                tx.send(Message::InterfaceDown(ip)).await.unwrap();
                            }
                        }
                        Err(_) => ()
                    }
                }
            }
        }
    })
}
