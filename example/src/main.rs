use matrix_bot::MatrixBot;
use std::env;

pub mod plugins {
    pub mod uuid;
}
use crate::plugins::uuid::UuidHandler;

#[tokio::main]
async fn main() {
    let config_path = match env::args().nth(1) {
        Some(a)=> a,
        None => {
            match directories::ProjectDirs::from("ca", "ahal", "testbot") {
                Some(dirs) => {
                    let path = dirs.config_dir().join("config.toml");
                    String::from(path.to_str().unwrap())
                },
                None => String::from("config.toml")
            }
        }
    };

    let mut bot = MatrixBot::new(&config_path)
        .await
        .unwrap();
    bot.add_handler(UuidHandler {});
    bot.run().await.unwrap();
}
