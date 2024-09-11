use super::{PGActor, PGMessage, PGQueueMessage};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct PGActorQueue {
    pg_actor_tx: mpsc::Sender<PGMessage>,
    pg_queue_rx: mpsc::Receiver<PGQueueMessage>,
}

impl PGActorQueue {
    pub async fn new() -> (Self, mpsc::Sender<PGQueueMessage>) {
        let (mut pg_actor, pg_actor_tx) = PGActor::new().await;
        tokio::spawn(async move {
            pg_actor.run().await;
        });

        let (pg_queue_tx, pg_queue_rx) = mpsc::channel(200);
        let queue = PGActorQueue {
            pg_actor_tx,
            pg_queue_rx,
        };
        (queue, pg_queue_tx)
    }

    pub async fn run(&mut self) {
        while let Some(first_msg) = self.pg_queue_rx.recv().await {
            let mut batch = Vec::new();

            batch.push(first_msg);

            while let Ok(msg) = self.pg_queue_rx.try_recv() {
                batch.push(msg);
                if batch.len() >= 100 {
                    break;
                }
            }

            if let Err(e) = self
                .pg_actor_tx
                .send(PGMessage::GetUserBatch { respond_to: batch })
                .await
            {
                eprintln!("Failed to send batched request to actor: {:?}", e);
            }
        }

        eprintln!("Queue has been closed.");
    }
}
