use axum::response::Html;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct RootActor {
    receiver: mpsc::Receiver<RootMessage>,
}

pub enum RootQueueMessage {
    GetRoot {
        respond_to: oneshot::Sender<Html<&'static str>>,
    },
}

pub enum RootMessage {
    GetRootBatch {
        respond_to: Vec<oneshot::Sender<Html<&'static str>>>,
    },
}

impl RootActor {
    pub fn new() -> (Self, mpsc::Sender<RootMessage>) {
        let (sender, receiver) = mpsc::channel(100);
        let actor = RootActor { receiver };
        (actor, sender)
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, msg: RootMessage) {
        match msg {
            RootMessage::GetRootBatch { respond_to } => {
                // Send the same response to all batched requests
                for responder in respond_to {
                    let _ = responder.send(Html("Hello, batched world!"));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RootActorQueue {
    root_actor_tx: mpsc::Sender<RootMessage>, // Communicates with the actor
    root_queue_rx: mpsc::Receiver<RootQueueMessage>, // Receives individual requests
}

impl RootActorQueue {
    pub fn new() -> (Self, mpsc::Sender<RootQueueMessage>) {
        let (mut root_actor, root_actor_tx) = RootActor::new();
        tokio::spawn(async move {
            root_actor.run().await;
        });
        // (queue, queue_sender)
        let (root_queue_tx, root_queue_rx) = mpsc::channel(100); // Channel for incoming requests
        let queue = RootActorQueue {
            root_actor_tx,
            root_queue_rx,
        };
        (queue, root_queue_tx)
    }

    pub async fn run(&mut self) {
        // Top-level loop to continuously process batches
        while let Some(first_msg) = self.root_queue_rx.recv().await {
            let mut batch = Vec::new();

            // Extract the oneshot sender from the first message and add it to the batch
            let RootQueueMessage::GetRoot { respond_to } = first_msg;
            batch.push(respond_to);

            // Try to take up to 4 more messages from the queue non-blockingly
            while let Ok(RootQueueMessage::GetRoot { respond_to }) = self.root_queue_rx.try_recv() {
                batch.push(respond_to);
                println!("Batched up another: {}", batch.len());
                if batch.len() >= 5 {
                    break; // Stop if we've reached the batch limit
                }
            }

            // Send the batch to the actor
            if let Err(e) = self
                .root_actor_tx
                .send(RootMessage::GetRootBatch { respond_to: batch })
                .await
            {
                eprintln!("Failed to send batched request to actor: {:?}", e);
            }
        }

        // Channel closed, exit the run function
        eprintln!("Queue has been closed.");
    }
}
