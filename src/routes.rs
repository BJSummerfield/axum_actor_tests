use super::pg_actor::PGMessage;
use super::root_actor::RootMessage;
use super::state::AppState;

use axum::{
    extract::State,
    response::{Html, Json},
};
use tokio::sync::oneshot;

pub struct Routes;

impl Routes {
    pub async fn get_root_handler(
        State(AppState { root_receiver, .. }): State<AppState>,
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

    pub async fn get_user_handler(
        State(AppState { pg_receiver, .. }): State<AppState>,
    ) -> Json<serde_json::Value> {
        let (send, recv) = oneshot::channel();
        let msg = PGMessage::GetUser { respond_to: send };

        // Send the message to the actor
        if let Err(_) = pg_receiver.send(msg).await {
            // Return a JSON error response
            return Json(serde_json::json!({
                "error": "Actor task has been killed."
            }));
        }

        match recv.await {
            Ok(user) => Json(serde_json::json!(user)), // Return the user in JSON format
            Err(_) => Json(serde_json::json!({
                "error": "Failed to get response from actor."
            })),
        }
    }
}
