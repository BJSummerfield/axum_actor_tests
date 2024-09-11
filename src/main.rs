use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::Serialize;
use tokio_postgres::NoTls;

#[derive(Serialize)]
struct User {
    id: i32,
    username: String,
}

#[tokio::main]
async fn main() {
    let database_url = "postgresql://postgres:password@localhost:5432/mydb";

    let manager = PostgresConnectionManager::new_from_stringlike(&database_url, NoTls).unwrap();
    let pool = Pool::builder()
        .max_size(100)
        .min_idle(Some(100))
        .build(manager)
        .await
        .unwrap();
    println!("Connection pool created");

    let app = Router::new()
        .route("/", get(|| async { Html("Hello, world!") }))
        .route("/users", get(get_users))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn get_users(
    State(pool): State<ConnectionPool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query("SELECT id, username FROM users", &[])
        .await
        .map_err(internal_error)?;

    let users: Vec<User> = rows
        .iter()
        .map(|row| User {
            id: row.get(0),
            username: row.get(1),
        })
        .collect();

    Ok(Json(users))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    println!("Internal error: {}", err);
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
