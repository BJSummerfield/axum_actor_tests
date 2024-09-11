use axum::{extract::State, response::Html, routing::get, Router};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
struct RootActor {
    receiver: mpsc::Receiver<RootMessage>,
}

enum RootMessage {
    GetRoot {
        respond_to: oneshot::Sender<Html<&'static str>>,
    },
}

impl RootActor {
    fn new() -> (Self, mpsc::Sender<RootMessage>) {
        let (sender, receiver) = mpsc::channel(8);
        let actor = RootActor { receiver };
        (actor, sender)
    }

    async fn run(&mut self) {
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

#[tokio::main]
async fn main() {
    let (mut root_actor, tx) = RootActor::new();

    tokio::spawn(async move {
        root_actor.run().await;
    });

    let app = Router::new().route("/", get(handler)).with_state(tx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(tx): State<mpsc::Sender<RootMessage>>) -> Html<&'static str> {
    let (send, recv) = oneshot::channel();
    let msg = RootMessage::GetRoot { respond_to: send };

    // Send the message to the actor
    if let Err(_) = tx.send(msg).await {
        return Html("Actor task has been killed.");
    }

    recv.await
        .unwrap_or_else(|_| Html("Failed to get response from actor."))
}
