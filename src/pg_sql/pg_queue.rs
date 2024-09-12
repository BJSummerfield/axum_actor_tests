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

        let (pg_queue_tx, pg_queue_rx) = mpsc::channel(100);
        let queue = PGActorQueue {
            pg_actor_tx,
            pg_queue_rx,
        };
        (queue, pg_queue_tx)
    }

    pub async fn run(&mut self) {
        let mut batch = Vec::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50)); // Adjust this value for how long you want to wait for messages

        loop {
            tokio::select! {
                msg = self.pg_queue_rx.recv() => {
                    if let Some(msg) = msg {
                        batch.push(msg);

                        if batch.len() >= 40 {
                            if let Err(e) = self
                                .pg_actor_tx
                                .send(PGMessage::GetUserBatch { respond_to: batch.split_off(0) })
                                .await
                            {
                                eprintln!("Failed to send batched request to actor: {:?}", e);
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ = interval.tick() => {
                    if !batch.is_empty() {
                    println!("batch len: {}", batch.len());
                        if let Err(e) = self
                            .pg_actor_tx
                            .send(PGMessage::GetUserBatch { respond_to: batch.split_off(0) })
                            .await
                        {
                            eprintln!("Failed to send batched request to actor: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}
