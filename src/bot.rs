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
use tokio::time::{sleep, Duration};
use url::Url;

pub struct MatrixBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    pub client: Client,
    handlers: Vec<Box<dyn MessageHandler + Send + Sync>>,
}

impl MatrixBot {
    pub async fn new(
        homeserver: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, matrix_sdk::Error> {
        tracing_subscriber::fmt::init();

        // the location for `JsonStore` to save files to
        let mut home = dirs::home_dir().expect("no home directory found");
        home.push("testbot");

        let client_config = ClientConfig::new().store_path(home);

        let homeserver = Url::parse(&homeserver).expect("Couldn't parse the homeserver URL");
        // create a new Client with the given homeserver url and config
        let client = Client::new_with_config(homeserver, client_config).unwrap();

        client
            .login(username, password, None, Some("testbot"))
            .await
            .unwrap();

        println!("logged in as {}", username);

        Ok(Self {
            client,
            handlers: vec![],
        })
    }

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
                    room.room_id(), err, delay
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
    use matrix_sdk_test::test_json;
    use mockito::mock;

    #[tokio::test]
    async fn login() {
        let homeserver = mockito::server_url();
        let _m = mock("POST", "/_matrix/client/r0/login")
            .with_status(200)
            .with_body(test_json::LOGIN.to_string())
            .create();

        let bot = MatrixBot::new(&homeserver, "user", "password")
            .await
            .unwrap();
        let logged_in = bot.client.logged_in().await;
        assert!(logged_in, "Bot should be logged in");
    }
}
