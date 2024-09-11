use super::{root::RootQueueMessage, state::AppState};
use crate::pg_sql::PGQueueMessage;

use axum::{
    extract::State,
    response::{Html, Json},
};
use serde_json::json;
use tokio::sync::oneshot;

pub struct Routes;

impl Routes {
    pub async fn get_root_handler(
        State(AppState { root_tx, .. }): State<AppState>,
    ) -> Html<&'static str> {
        let (send, recv) = oneshot::channel();
        let msg = RootQueueMessage::GetRoot { respond_to: send };

        if let Err(_) = root_tx.send(msg).await {
            return Html("Actor task has been killed.");
        }

        recv.await
            .unwrap_or_else(|_| Html("Failed to get response from actor."))
    }

    pub async fn get_user_handler(
        State(AppState { pg_tx, .. }): State<AppState>,
    ) -> Json<serde_json::Value> {
        let (send, recv) = oneshot::channel();
        let msg = PGQueueMessage::GetUser {
            respond_to: send,
            user_id: 1,
        };

        if let Err(_) = pg_tx.send(msg).await {
            return Json(json!({
                "error": "Actor task has been killed."
            }));
        }

        match recv.await {
            Ok(Ok(user)) => Json(json!(user)),
            Ok(Err(e)) => Json(json!({
                "error": format!("Failed to get user: {}", e)
            })),
            Err(_) => Json(json!({
                "error": "Failed to receive response from actor."
            })),
        }
    }
}
