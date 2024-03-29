use anyhow::Result;
use matrix_bot::{config::MatrixBotConfig, MatrixBot};
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    Client,
};
use std::env;
use std::path::PathBuf;

pub mod plugins {
    pub mod uuid;
}
use crate::plugins::uuid::UuidHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let config = MatrixBotConfig::new(env::args().nth(1));

    let _statedir = match config.statedir {
        Some(a) => PathBuf::from(a),
        None => {
            match directories::ProjectDirs::from("ca", "ahal", "matrix-bot") {
                Some(dirs) => dirs.data_dir().to_path_buf(),
                None => {
                    // the location for `JsonStore` to save files to
                    let mut home = dirs::home_dir().expect("no home directory found");
                    home.push(".matrix-bot-state");
                    home
                }
            }
        }
    };

    let client_builder = Client::builder().homeserver_url(&config.homeserver);

    let client = client_builder.build().await.unwrap();

    client
        .login(&config.username, &config.password, None, Some("testbot"))
        .await?;

    println!("logged in as {}", &config.username);

    // An initial sync to set up state and so our bot doesn't respond to old messages.
    // If the `StateStore` finds saved state in the location given the initial sync will
    // be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await.unwrap();

    let mut bot = MatrixBot::new().await.unwrap();
    bot.add_handler(&UuidHandler {});

    client.register_event_handler({
        let bot = bot.clone();
        move |event: OriginalSyncRoomMessageEvent, room: Room| {
            let bot = bot.clone();
            async move {
                bot.on_room_message(&room, &event).await;
            }
        }
    }).await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    // this keeps state from the server streaming in to MatrixBot via the EventHandler trait
    client.sync(settings).await;
    Ok(())
}
