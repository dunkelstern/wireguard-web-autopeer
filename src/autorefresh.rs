use crate::state::{messages::Message, structs::Timeout};
use tokio::{sync::mpsc::Sender, task::JoinHandle, select};
use tokio_util::sync::CancellationToken;


pub fn autorefresh(tx: Sender<Message>, cancel: CancellationToken, timeout: Timeout) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            select! {
                // cancelled, break loop, exit task
                _ = cancel.cancelled() => {
                    break;
                }
                // 60 seconds timeout, send refresh message
                _ = tokio::time::sleep(std::time::Duration::from_secs(timeout.into())) => {
                    let _ = tx.send(Message::RefreshPeers).await.unwrap();
                }
            }
        }
    })
}
