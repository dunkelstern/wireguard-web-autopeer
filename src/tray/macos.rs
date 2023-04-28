use tray_item::TrayItem;

pub fn initialize_tray() -> Option<TrayItem> {
    let mut tray = TrayItem::new("Wireguard-Web-Autopeer", "wireguard-web-autopeer").unwrap();

    tray.add_label("Wireguard-Web").unwrap();

    // FIXME: Add a refresh menu item
    //
    // tray.add_menu_item("Refresh Peers", || {
    // }).unwrap();
    
    let mut inner = tray.inner_mut();
    inner.add_quit_item("Quit");
    inner.display();
    Some(tray)
}
