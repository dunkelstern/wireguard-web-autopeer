use default_net;

use futures::StreamExt;
use if_watch::{tokio::IfWatcher, IfEvent};

use crate::storage::{Changed, State};
use crate::network_interface::{if_up, if_down};

pub type NetworkChangeCallback = fn(state: State) -> State;


pub async fn monitor(update: NetworkChangeCallback) {
    let mut state = State::new();

    // watch interfaces go up or down
    tokio::spawn(async {
        let mut set = IfWatcher::new().unwrap();
        loop {
            let event = set.select_next_some().await;
            if let Ok(ev) = event {
                let interfaces = default_net::get_interfaces();
                state = match ev {
                    IfEvent::Up(ip) => {
                        match if_up(ip, interfaces, &state.if_database) {
                            Changed::ValueChanged(db) => {
                                state.if_database = db;
                                update(state)
                            }
                            Changed::ValueUnchanged(_db) => state
                        }
                    }
                    IfEvent::Down(ip) => {
                        match if_down(ip, interfaces, &state.if_database) {
                            Changed::ValueChanged(db) => {
                                state.if_database = db;
                                update(state)
                            }
                            Changed::ValueUnchanged(_db) => state
                        }
                    }
                }
            }
        }
    });
}