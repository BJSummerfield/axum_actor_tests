use super::{PGMessage, PGQueueMessage, User};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tokio_postgres::NoTls;

#[derive(Debug)]
pub struct PGActor {
    receiver: mpsc::Receiver<PGMessage>,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PGActor {
    pub async fn new() -> (Self, mpsc::Sender<PGMessage>) {
        let database_url = "postgresql://postgres:password@localhost:5432/mydb";

        let manager = PostgresConnectionManager::new_from_stringlike(&database_url, NoTls).unwrap();
        let connection_pool = Pool::builder().build(manager).await.unwrap();
        println!("Connection pool created");

        let (sender, receiver) = mpsc::channel(1);
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

    fn log_pool_state(connection_pool: &Pool<PostgresConnectionManager<NoTls>>) {
        let state = connection_pool.state();
        let active_connections = state.connections - state.idle_connections;
        println!(
            "Connection pool state: {} active connections, {} idle connections",
            active_connections, state.idle_connections
        );
    }
    async fn handle_message(
        msg: PGMessage,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) {
        match msg {
            PGMessage::GetUserBatch { respond_to } => {
                let mut unique_user_ids = HashSet::new();
                let mut requests_by_user_id = HashMap::new();

                for msg in respond_to.into_iter() {
                    let PGQueueMessage::GetUser {
                        user_id,
                        respond_to,
                    } = msg;
                    {
                        unique_user_ids.insert(user_id);

                        requests_by_user_id
                            .entry(user_id)
                            .or_insert_with(Vec::new)
                            .push(respond_to);
                    }
                }

                let unique_user_ids: Vec<i32> = unique_user_ids.into_iter().collect();

                if let Ok(users) = PGActor::get_users(unique_user_ids, connection_pool).await {
                    let mut user_map: HashMap<i32, User> = HashMap::new();
                    for user in users {
                        user_map.insert(user.id, user);
                    }

                    for (user_id, senders) in requests_by_user_id {
                        if let Some(user) = user_map.get(&user_id) {
                            for sender in senders {
                                let _ = sender.send(Ok(user.clone()));
                            }
                        } else {
                            for sender in senders {
                                let _ = sender.send(Err(Box::new(std::io::Error::new(
                                    std::io::ErrorKind::NotFound,
                                    "User not found",
                                ))));
                            }
                        }
                    }
                }
            }
        }
    }

    async fn get_users(
        user_ids: Vec<i32>,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = connection_pool.get().await?;
        let rows = conn
            .query(
                "SELECT id, username FROM users WHERE id = ANY($1)",
                &[&user_ids],
            )
            .await?;

        let users: Vec<User> = rows
            .into_iter()
            .map(|row| User {
                id: row.get(0),
                username: row.get(1),
            })
            .collect();

        Ok(users)
    }
}
