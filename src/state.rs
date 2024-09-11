use super::root_actor::{RootActor, RootMessage};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub root_receiver: mpsc::Sender<RootMessage>,
}

impl AppState {
    pub fn new() -> Self {
        let (mut root_actor, tx) = RootActor::new();
        tokio::spawn(async move {
            root_actor.run().await;
        });
        AppState { root_receiver: tx }
    }
}
