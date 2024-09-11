use axum::response::Html;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct RootActor {
    receiver: mpsc::Receiver<RootMessage>,
}

pub enum RootMessage {
    GetRoot {
        respond_to: oneshot::Sender<Html<&'static str>>,
    },
}

impl RootActor {
    pub fn new() -> (Self, mpsc::Sender<RootMessage>) {
        let (sender, receiver) = mpsc::channel(8);
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
            RootMessage::GetRoot { respond_to } => {
                let _ = respond_to.send(Html("Hello, world!"));
            }
        }
    }
}
