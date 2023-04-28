
use tokio::sync::mpsc::{Sender, channel};
use std::{fs, thread};
use xdg;

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


        let icon_data = r###"
        <?xml version="1.0" encoding="UTF-8" standalone="no"?>
        <svg version="1.1" id="Layer_1" x="0px" y="0px" width="111.811px" height="122.88px" viewBox="0 0 111.811 122.88" enable-background="new 0 0 111.811 122.88">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M 55.731156,4.1397753 C 74.616297,14.911946 85.958318,22.074347 100.8499,20.712694 103.45068,73.315981 92.824497,106.29204 56.62689,118.74023 22.000256,108.19428 9.3530215,75.482833 10.811409,19.914865 28.292711,20.829697 38.543971,15.915348 55.731156,4.1397753 Z m 0.414576,15.0162037 c 13.097848,8.30166 21.245655,14.071641 31.400011,13.144833 1.773465,35.870986 -7.788507,62.5826 -31.263661,71.423208 -0.148326,-0.0543 -0.293889,-0.11055 -0.441294,-0.16583 V 19.389063 Z M 55.751424,11.849762 C 72.752739,22.625983 81.137604,26.529025 93.92377,26.529025 96.225129,73.091474 86.060409,99.594446 56.068578,111.44736 27.317331,100.03958 16.464329,75.360306 17.755964,26.171285 c 14.705085,0 23.285371,-2.893036 37.99546,-14.321523 z" id="path57" fill="#ffffff" style="stroke-width:0.921281;fill:#ffffff" />
        </svg>
        "###;

        if let Ok(xdg_dir) = xdg::BaseDirectories::new() {
            if let Ok(filename) = xdg_dir.place_data_file("icons/wireguard-web-autopeer.svg") {
                if !filename.exists() {
                    fs::write(filename, icon_data).expect("Unable to save icon file.");
                }
            }
        }

        let (tx, rx) = channel::<Message>(1);
        let tray_service = ksni::TrayService::new(WireguardWebTray{events, enabled: true});
        let tray = tray_service.handle();

        thread::spawn(|| {
            if let Err(err) = tray_service.run() {
                error!("Can not start systray: {:?}", err);
            }
        });

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