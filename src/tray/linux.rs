
use tokio::sync::mpsc::{Sender, channel};

use crate::state::messages::Message;

use super::Tray;
use ksni;

#[derive(Debug)]
pub struct WireguardWebTray {
    events: Sender<Message>,
    enabled: bool
}

impl ksni::Tray for WireguardWebTray {
    fn icon_name(&self) -> String {
        "wireguard-web-autopeer".into()
    }
    
    fn title(&self) -> String {
        "WireguardWeb".into()
    }
    
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            CheckmarkItem {
                label: "Enabled".into(),
                checked: self.enabled,
                activate: Box::new(|this: &mut Self| {
                    this.enabled = !this.enabled;
                    if this.enabled {
                        this.events.try_send(Message::Resume).expect("Failed to send resume message to main loop");
                    } else {
                        this.events.try_send(Message::Suspend).expect("Failed to send suspend message to main loop");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Refresh".into(),
                icon_name: "view-refresh-symbolic".into(),
                activate: Box::new(|this: &mut Self| { this.events.try_send(Message::RefreshPeers).expect("Failed to send refresh message to main loop"); }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|this: &mut Self| { this.events.try_send(Message::Quit).expect("Failed to send quit message to main loop"); }),
                ..Default::default()
            }
            .into(),
        ]
    }
    fn id(&self) -> String {
        "mytray".to_string()
    }
}


impl Tray {
    pub fn try_new(events: Sender<Message>) -> (Option<Self>, Option<Sender<Message>>) {
        debug!("Creating Tray icon...");

        let (tx, rx) = channel::<Message>(1);
        let tray_service = ksni::TrayService::new(WireguardWebTray{events, enabled: true});
        let tray = tray_service.handle();

        tray_service.spawn();

        (Some(Self{
            tray,
            rx
        }), Some(tx))
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                Message::Quit => {
                    self.tray.shutdown();
                    debug!("Received quit message!")
                }
                Message::Suspend => {
                    self.tray.update(|tray: &mut WireguardWebTray| {
                        tray.enabled = false;
                    });
                    debug!("Received suspend message!")
                }
                Message::Resume => {
                    self.tray.update(|tray: &mut WireguardWebTray| {
                        tray.enabled = true;
                    });
                    debug!("Received resume message!")
                }
                _ => ()
            }
        }
        debug!("Tray loop exited!");
    }
    
}