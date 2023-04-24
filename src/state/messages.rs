use if_watch::IpNet;

#[derive(Debug, Clone, PartialEq)]
pub enum Message{
    InterfaceUp(IpNet),
    InterfaceDown(IpNet),
    RefreshPeers,
    Suspend,
    Resume,
    Quit
}