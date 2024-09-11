use axum::response::Html;
use tokio::sync::oneshot;

pub enum RootMessage {
    GetRootBatch {
        respond_to: Vec<oneshot::Sender<Html<&'static str>>>,
    },
}

pub enum RootQueueMessage {
    GetRoot {
        respond_to: oneshot::Sender<Html<&'static str>>,
    },
}
