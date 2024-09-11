use super::{RootActor, RootMessage, RootQueueMessage};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct RootActorQueue {
    root_actor_tx: mpsc::Sender<RootMessage>,
    root_queue_rx: mpsc::Receiver<RootQueueMessage>,
}

impl RootActorQueue {
    pub fn new() -> (Self, mpsc::Sender<RootQueueMessage>) {
        let (mut root_actor, root_actor_tx) = RootActor::new();
        tokio::spawn(async move {
            root_actor.run().await;
        });

        let (root_queue_tx, root_queue_rx) = mpsc::channel(100);
        let queue = RootActorQueue {
            root_actor_tx,
            root_queue_rx,
        };
        (queue, root_queue_tx)
    }

    pub async fn run(&mut self) {
        while let Some(first_msg) = self.root_queue_rx.recv().await {
            let mut batch = Vec::new();

            let RootQueueMessage::GetRoot { respond_to } = first_msg;
            batch.push(respond_to);

            while let Ok(RootQueueMessage::GetRoot { respond_to }) = self.root_queue_rx.try_recv() {
                batch.push(respond_to);
                if batch.len() >= 10 {
                    break;
                }
            }

            if let Err(e) = self
                .root_actor_tx
                .send(RootMessage::GetRootBatch { respond_to: batch })
                .await
            {
                eprintln!("Failed to send batched request to actor: {:?}", e);
            }
        }

        eprintln!("Queue has been closed.");
    }
}
