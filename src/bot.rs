use std::{env, process::exit};
use tokio::time::{delay_for, Duration};

mod handler;
use crate::handler::{HandleResult, MessageHandler};

pub mod apps {
    pub mod uuid;
}

use matrix_sdk::{
    self, async_trait,
    events::{
        room::{
            member::MemberEventContent,
            message::MessageEventContent,
        },
        StrippedStateEvent, SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use url::Url;

pub struct MatrixBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    client: Client,
    handlers: Vec<Box<dyn MessageHandler + Send + Sync>>,
}

impl MatrixBot {
    pub fn new() -> Result<Self, matrix_sdk::Error> {
        tracing_subscriber::fmt::init();
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let (homeserver_url, username, password) =
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

        // the location for `JsonStore` to save files to
        let mut home = dirs::home_dir().expect("no home directory found");
        home.push("testbot");

        let store = JsonStore::open(&home)?;
        let client_config = ClientConfig::new().state_store(Box::new(store));

        let homeserver_url = Url::parse(&homeserver_url).expect("Couldn't parse the homeserver URL");
        // create a new Client with the given homeserver url and config
        let client = Client::new_with_config(homeserver_url, client_config).unwrap();

        runtime.block_on(async {
            client
                .login(&username, &password, None, Some("testbot"))
                .await;
        });

        println!("logged in as {}", username);


        Ok(Self {
            client,
            handlers: vec![],
        })
    }

    pub fn run(self) -> Result<(), matrix_sdk::Error> {
        let mut client = self.client.clone();
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            // An initial sync to set up state and so our bot doesn't respond to old messages.
            // If the `StateStore` finds saved state in the location given the initial sync will
            // be skipped in favor of loading state from the store
            client.sync_once(SyncSettings::default()).await.unwrap();
            client
                .add_event_emitter(Box::new(self))
                .await;

            // since we called `sync_once` before we entered our sync loop we must pass
            // that sync token to `sync`
            let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
            // this keeps state from the server streaming in to MatrixBot via the EventEmitter trait
            client.sync(settings).await;
        });
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
impl EventEmitter for MatrixBot {
    async fn on_stripped_state_member(
        &self,
        room: SyncRoom,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        if room_member.state_key != self.client.user_id().await.unwrap() {
            return;
        }

        if let SyncRoom::Invited(room) = room {
            let room = room.read().await;
            println!("Autojoining room {}", room.room_id);
            let mut delay = 2;

            while let Err(err) = self.client.join_room_by_id(&room.room_id).await {
                // retry autojoin due to synapse sending invites, before the
                // invited user can join for more information see
                // https://github.com/matrix-org/synapse/issues/4345
                eprintln!(
                    "Failed to join room {} ({:?}), retrying in {}s",
                    room.room_id, err, delay
                );

                delay_for(Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    eprintln!("Can't join room {} ({:?})", room.room_id, err);
                    break;
                }
            }
            println!("Successfully joined room {}", room.room_id);
        }
    }

    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        for handler in self.handlers.iter() {
            let val = handler.handle_message(&self, &room, event).await;
            match val {
                HandleResult::Continue => continue,
                HandleResult::Stop => break,
            }
        }
    }

}
