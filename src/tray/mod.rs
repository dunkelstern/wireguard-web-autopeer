#[cfg(target_os = "linux")]
use ksni::Handle;

#[cfg(target_os = "linux")]
use self::linux::WireguardWebTray;


#[cfg(not(target_os = "linux"))]
use tray_item::TrayItem;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

use crate::state::messages::Message;
use tokio::sync::mpsc::Receiver;


pub struct Tray {
    #[cfg(target_os = "linux")]
    tray: Handle<WireguardWebTray>,

    #[cfg(not(target_os = "linux"))]
    tray: TrayItem,
    
    rx: Receiver<Message>,
}
