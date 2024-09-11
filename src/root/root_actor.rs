use super::RootMessage;
use axum::response::Html;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct RootActor {
    receiver: mpsc::Receiver<RootMessage>,
}

impl RootActor {
    pub fn new() -> (Self, mpsc::Sender<RootMessage>) {
        let (sender, receiver) = mpsc::channel(1);
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
