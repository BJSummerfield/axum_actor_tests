use super::pg_actor::{PGActor, PGMessage};
use super::root_actor::{RootActor, RootMessage};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub root_receiver: mpsc::Sender<RootMessage>,
    pub pg_receiver: mpsc::Sender<PGMessage>,
}

impl AppState {
    pub async fn new() -> Self {
        let (mut root_actor, root_tx) = RootActor::new();
        tokio::spawn(async move {
            root_actor.run().await;
        });

        let (mut pg_actor, pg_tx) = PGActor::new().await;
        tokio::spawn(async move {
            pg_actor.run().await;
        });
        AppState {
            root_receiver: root_tx,
            pg_receiver: pg_tx,
        }
    }
}
