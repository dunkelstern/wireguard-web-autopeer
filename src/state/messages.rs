
#[derive(Debug, Clone, PartialEq)]
pub enum Message{
    InterfaceUp(String),
    InterfaceDown(String),
    RefreshPeers,
    Suspend,
    Resume,
    Quit
}