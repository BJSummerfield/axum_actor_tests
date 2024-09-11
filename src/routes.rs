use super::root_actor::RootMessage;
use super::state::AppState;

use axum::{extract::State, response::Html};
use tokio::sync::oneshot;

pub struct Routes;

impl Routes {
    pub async fn get_root_handler(
        State(AppState { root_receiver }): State<AppState>,
    ) -> Html<&'static str> {
        let (send, recv) = oneshot::channel();
        let msg = RootMessage::GetRoot { respond_to: send };

        // Send the message to the actor
        if let Err(_) = root_receiver.send(msg).await {
            return Html("Actor task has been killed.");
        }

        recv.await
            .unwrap_or_else(|_| Html("Failed to get response from actor."))
    }
}
