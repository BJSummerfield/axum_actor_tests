use super::{
    pg_sql::{PGActorQueue, PGQueueMessage},
    root::{RootActorQueue, RootQueueMessage},
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub root_tx: mpsc::Sender<RootQueueMessage>,
    pub pg_tx: mpsc::Sender<PGQueueMessage>,
}

impl AppState {
    pub async fn new() -> Self {
        let (mut root_actor, root_tx) = RootActorQueue::new();
        tokio::spawn(async move {
            root_actor.run().await;
        });

        let (mut pg_actor, pg_tx) = PGActorQueue::new().await;
        tokio::spawn(async move {
            pg_actor.run().await;
        });
        AppState { root_tx, pg_tx }
    }
}
