use serde::Serialize;
use tokio::sync::oneshot;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
}

pub enum PGQueueMessage {
    GetUser {
        user_id: i32,
        respond_to: oneshot::Sender<Result<User, Box<dyn std::error::Error + Send + Sync>>>,
    },
}
pub enum PGMessage {
    GetUserBatch { respond_to: Vec<PGQueueMessage> },
}
