mod pg_sql;
mod root;
mod routes;
mod state;

use axum::{routing::get, Router};
use routes::Routes;
use state::AppState;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;

    let app = Router::new()
        .route("/", get(Routes::get_root_handler))
        .route("/users", get(Routes::get_user_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
