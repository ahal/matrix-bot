use matrix_bot::MatrixBot;
use std::{env, process::exit};

pub mod plugins {
    pub mod uuid;
}
use crate::plugins::uuid::UuidHandler;

#[tokio::main]
async fn main() {
    let (homeserver, username, password) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password>",
                    env::args().next().unwrap()
                );
                exit(1)
            }
        };

    let mut bot = MatrixBot::new(&homeserver, &username, &password)
        .await
        .unwrap();
    bot.add_handler(UuidHandler {});
    bot.run().await.unwrap();
}
