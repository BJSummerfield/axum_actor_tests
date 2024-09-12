use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use tokio_postgres::NoTls;

#[derive(Serialize)]
pub struct User {
    id: i32,
    username: String,
}

#[derive(Debug)]
pub struct PGActor {
    receiver: mpsc::Receiver<PGMessage>,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
}

pub enum PGMessage {
    GetUser { respond_to: oneshot::Sender<User> },
}

impl PGActor {
    pub async fn new() -> (Self, mpsc::Sender<PGMessage>) {
        let database_url = "postgresql://postgres:password@localhost:5432/mydb";

        let manager = PostgresConnectionManager::new_from_stringlike(&database_url, NoTls).unwrap();
        let connection_pool = Pool::builder()
            .max_size(80)
            .min_idle(Some(5))
            .build(manager)
            .await
            .unwrap();
        println!("Connection pool created");

        let (sender, receiver) = mpsc::channel(8);
        let actor = PGActor {
            receiver,
            connection_pool,
        };
        (actor, sender)
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            let pool = self.connection_pool.clone();
            tokio::spawn(async move {
                PGActor::handle_message(msg, pool).await;
            });
        }
    }

    async fn handle_message(
        msg: PGMessage,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) {
        match msg {
            PGMessage::GetUser { respond_to } => {
                let result = PGActor::get_user(connection_pool).await;
                let _ = respond_to.send(result.unwrap());
            }
        }
    }

    async fn get_user(
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let conn = connection_pool.get().await?;
        let row = conn
            .query_one("SELECT id, username FROM users WHERE id = 1", &[])
            .await?;
        Ok(User {
            id: row.get(0),
            username: row.get(1),
        })
    }
}
