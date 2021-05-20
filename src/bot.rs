//! This crate implements a framework for [Matrix](https://matrix.org/) bots. It provides
//! encrpytion support (via [matrix-rust-sdk](https://docs.rs/matrix-sdk/latest/matrix_sdk/)),
//! autojoin on invite, and more. Actual functionality is up to consumers to provide via a plugin
//! system.
//!
//! Example `main.rs`:
//! ```rust
//! use matrix_bot::MatrixBot;
//! use std::env;
//! 
//! pub mod plugins {
//!     pub mod uuid;
//! }
//! use crate::plugins::uuid::UuidHandler;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let config_path = match env::args().nth(1) {
//!         Some(a)=> a,
//!         None => {
//!             match directories::ProjectDirs::from("ca", "ahal", "testbot") {
//!                 Some(dirs) => {
//!                     let path = dirs.config_dir().join("config.toml");
//!                     String::from(path.to_str().unwrap())
//!                 },
//!                 None => String::from("config.toml")
//!             }
//!         }
//!     };
//! 
//!     let mut bot = MatrixBot::new(&config_path)
//!         .await
//!         .unwrap();
//!     bot.add_handler(UuidHandler {});
//!     bot.run().await.unwrap();
//! }
//! ```
//!
//! # Configuration
//!
//! The `MatrixBot` struct expects a path to a [toml](https://github.com/toml-lang/toml)
//! configuration file. This file can contain the following values:
//!
//! ```toml
//! # url to homeserver of user (required)
//! homeserver = https://example.org
//!
//! # bot's username (required)
//! username = robocop
//!
//! # bot's password (required)
//! password = hunter2
//!
//! # path to directory to store state (optional, default's to a platform dependent [data
//! dir](https://docs.rs/directories/3.0.2/directories/struct.ProjectDirs.html#method.data_dir))
//! statedir = path/to/state
//! ```

use std::fs;
use std::path::PathBuf;

pub mod handler;
use crate::handler::{HandleResult, MessageHandler};

use matrix_sdk::{
    self, async_trait,
    events::{
        room::{member::MemberEventContent, message::MessageEventContent},
        StrippedStateEvent, SyncMessageEvent,
    },
    room::Room,
    Client, ClientConfig, EventHandler, SyncSettings,
};
use serde::Deserialize;
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Deserialize, Default)]
struct MatrixBotConfig<'a> {
    homeserver: &'a str,
    username: &'a str,
    password: &'a str,
    statedir: Option<&'a str>,
}

impl<'a> MatrixBotConfig<'a> {
    fn from_config(content: &'a str) -> MatrixBotConfig<'a> {
        let config: MatrixBotConfig = toml::from_str(content).unwrap();
        config
    }
}

/// Bot responsible for registering handlers and listening for messages in a loop upon a call to
/// `run`.
pub struct MatrixBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    pub client: Client,
    handlers: Vec<Box<dyn MessageHandler + Send + Sync>>,
}

impl MatrixBot {
    /// Create a new `MatrixBot` instance.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file.
    ///
    /// # Example
    ///
    /// ```
    /// let mut bot = MatrixBot::new("path/to/config.toml")
    ///     .await
    ///     .unwrap();
    /// ```
    pub async fn new(config_path: &str) -> Result<Self, matrix_sdk::Error> {
        let config_contents = fs::read_to_string(config_path).expect("Error reading config file!");
        let config = MatrixBotConfig::from_config(&config_contents);

        let statedir = match config.statedir {
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

        let client_config = ClientConfig::new().store_path(statedir);

        let homeserver = Url::parse(&config.homeserver).expect("Couldn't parse the homeserver URL");
        // create a new Client with the given homeserver url and config
        let client = Client::new_with_config(homeserver, client_config).unwrap();

        client
            .login(&config.username, &config.password, None, Some("testbot"))
            .await
            .unwrap();

        println!("logged in as {}", &config.username);

        Ok(Self {
            client,
            handlers: vec![],
        })
    }

    /// Listen for messages in a loop and forward them to any registered handlers.
    ///
    /// # Example
    ///
    /// ```
    /// bot.run().await.unwrap();
    /// ```
    pub async fn run(self) -> Result<(), matrix_sdk::Error> {
        let client = self.client.clone();

        // An initial sync to set up state and so our bot doesn't respond to old messages.
        // If the `StateStore` finds saved state in the location given the initial sync will
        // be skipped in favor of loading state from the store
        client.sync_once(SyncSettings::default()).await.unwrap();
        client.set_event_handler(Box::new(self)).await;

        // since we called `sync_once` before we entered our sync loop we must pass
        // that sync token to `sync`
        let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
        // this keeps state from the server streaming in to MatrixBot via the EventHandler trait
        client.sync(settings).await;
        Ok(())
    }

    /// Register a new handler.
    ///
    /// # Arguments
    ///
    /// * `handler` - An instance implementing the `MessageHandler` trait.
    ///
    /// # Example
    ///
    /// ```
    /// bot.add_handler(EchoHandler {})
    /// ```
    pub fn add_handler<M>(&mut self, handler: M)
    where
        M: handler::MessageHandler + 'static + Send + Sync,
    {
        self.handlers.push(Box::new(handler));
    }
}

#[async_trait]
impl EventHandler for MatrixBot {
    async fn on_stripped_state_member(
        &self,
        room: Room,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        if room_member.state_key != self.client.user_id().await.unwrap() {
            return;
        }

        if let Room::Invited(room) = room {
            println!("Autojoining room {}", room.room_id());
            let mut delay: u64 = 2;

            while let Err(err) = room.accept_invitation().await {
                // retry autojoin due to synapse sending invites, before the
                // invited user can join for more information see
                // https://github.com/matrix-org/synapse/issues/4345
                eprintln!(
                    "Failed to join room {} ({:?}), retrying in {}s",
                    room.room_id(),
                    err,
                    delay
                );

                sleep(Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    eprintln!("Can't join room {} ({:?})", room.room_id(), err);
                    break;
                }
            }
            println!("Successfully joined room {}", room.room_id());
        }
    }

    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        for handler in self.handlers.iter() {
            let val = handler.handle_message(&self, &room, event).await;
            match val {
                HandleResult::Continue => continue,
                HandleResult::Stop => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use matrix_sdk::{identifiers::room_id, Client};
    use matrix_sdk_test::{test_json, EventBuilder};
    use mockito::mock;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use url::form_urlencoded::byte_serialize;

    async fn get_client() -> Client {
        let homeserver = mockito::server_url();
        let _m = mock("POST", "/_matrix/client/r0/login")
            .with_status(200)
            .with_body(test_json::LOGIN.to_string())
            .create();

        let config = &format!(
            r#"
            homeserver = "{}"
            username = "user"
            password = "password"
            statedir = "{}"
        "#,
            homeserver,
            tempdir().unwrap().path().to_str().unwrap()
        );

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "{}", config).unwrap();

        let bot = MatrixBot::new(&path.to_str().unwrap()).await.unwrap();
        let client = bot.client.clone();
        client.set_event_handler(Box::new(bot)).await;
        client
    }

    #[tokio::test]
    async fn login() {
        let client = get_client().await;
        let logged_in = client.logged_in().await;
        assert!(logged_in, "Bot should be logged in");
    }

    #[tokio::test]
    async fn autojoin() {
        let client = get_client().await;
        let room_id = room_id!("!SVkFJHzfwvuaIEawgC:localhost");
        let room = client.get_invited_room(&room_id);
        assert!(room.is_none());

        let event = json!({
            "content": {
                "displayname": "example",
                "membership": "join"
            },
            "event_id": "$151800140517rfvjc:localhost",
            "membership": "join",
            "origin_server_ts": 0,
            "sender": "@example:localhost",
            "state_key": "@cheeky_monkey:matrix.org",
            "type": "m.room.member"
        });

        let sync_response_json = EventBuilder::default()
            .add_custom_invited_event(&room_id, event)
            .build_json_sync_response();

        let m1 = mock(
            "GET",
            mockito::Matcher::Regex(r"^/_matrix/client/r0/sync.*".to_string()),
        )
        .with_status(200)
        .with_body(sync_response_json.to_string())
        .create();

        let encoded_room_id: String = byte_serialize(room_id.as_bytes()).collect();
        let url = format!("/_matrix/client/r0/rooms/{}/join", encoded_room_id);
        let m2 = mock("POST", url.as_str())
            .with_status(200)
            .with_body(json!({ "room_id": &room_id }).to_string())
            .create();

        client.sync_once(SyncSettings::default()).await.unwrap();
        m1.assert();
        m2.assert();
    }
}
