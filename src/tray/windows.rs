use std::sync::mpsc;
use gtk;
use tray_item::TrayItem;

pub fn initialize_tray() -> Option<TrayItem> {
    let mut tray = TrayItem::new("Wireguard-Web-Autopeer", "wireguard-web-autopeer").unwrap();

    tray.add_label("Wireguard-Web").unwrap();
    
    // FIXME: Add a refresh menu item
    //
    // tray.add_menu_item("Refresh Peers", || {
    // }).unwrap();

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
    Some(tray)
}
