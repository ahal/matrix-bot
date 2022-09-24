use crate::handler::{HandleResult, MessageHandler};
use matrix_sdk::{
    room::{Joined, Room},
    ruma::{
        api::client::message::send_message_event,
        events::room::message::{
            MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
            TextMessageEventContent,
        },
    },
    Result as MatrixResult,
};

pub mod config;
pub mod handler;

#[derive(Clone)]
pub struct MatrixBot<'a> {
    handlers: Vec<&'a (dyn MessageHandler + Send + Sync)>,
}

impl<'a> MatrixBot<'a> {
    pub async fn new() -> Result<MatrixBot<'a>, matrix_sdk::Error> {
        Ok(Self { handlers: vec![] })
    }

    pub fn add_handler<M>(&mut self, handler: &'a M)
    where
        M: MessageHandler + Send + Sync,
    {
        self.handlers.push(handler);
    }

    pub async fn send(
        &self,
        room: &Joined,
        msg: String,
    ) -> MatrixResult<send_message_event::v3::Response> {
        let content = RoomMessageEventContent::text_plain(msg);
        room.send(content, None).await
    }

    pub async fn on_room_message(&self, room: &Room, event: &OriginalSyncRoomMessageEvent) {
        if let Room::Joined(room) = room {
            let msg = match &event.content.msgtype {
                MessageType::Text(TextMessageEventContent { body, .. }) => body,
                _ => return,
            };

            for handler in self.handlers.iter() {
                let val = handler.handle_message(&self, room, msg).await;
                match val {
                    HandleResult::Continue => continue,
                    HandleResult::Stop => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use matrix_sdk::{async_trait, config::SyncSettings, ruma::room_id, Client};
    use matrix_sdk_test::{test_json, EventBuilder};
    use mockito::mock;
    use serde_json::json;
    use url::form_urlencoded::byte_serialize;

    async fn get_client() -> Client {
        let homeserver = mockito::server_url();
        let _m = mock("POST", "/_matrix/client/r0/login")
            .with_status(200)
            .with_body(test_json::LOGIN.to_string())
            .create();

        let client_builder = Client::builder().homeserver_url(&homeserver);
        let client = client_builder.build().await.unwrap();
        client
            .login("user", "password", None, Some("testbot"))
            .await
            .unwrap();
        client
    }

    struct TestHandler {
        key: String,
    }

    #[async_trait]
    impl MessageHandler for TestHandler {
        async fn handle_message(
            &self,
            bot: &MatrixBot,
            room: &Joined,
            msg: &str,
        ) -> HandleResult {
            if msg == self.key {
                bot.send(room, format!("received: {}", msg)).await.unwrap();
                return HandleResult::Stop;
            }
            HandleResult::Continue
        }
    }

    #[tokio::test]
    async fn test_receive_message() {
        let mut bot = MatrixBot::new().await.unwrap();
        let handler1 = TestHandler {key: String::from("stop")};
        let handler2 = TestHandler {key: String::from("halt")};
        assert_eq!(bot.handlers.len(), 0);
        bot.add_handler(&handler1);
        bot.add_handler(&handler2);
        assert_eq!(bot.handlers.len(), 2);

        let client = get_client().await;
        client.register_event_handler({
            let bot = bot.clone();
            move |event: OriginalSyncRoomMessageEvent, room: Room| {
                let bot = bot.clone();
                async move {
                    bot.on_room_message(&room, &event).await;
                }
            }
        }).await;

        let _m = mock("GET", Matcher::Regex(r"^/_matrix/client/r0/sync\?.*$".to_owned()))
            .with_status(200)
            .with_body(test_json::SYNC.to_string())
            .match_header("authorization", "Bearer 1234")
            .create();

        let sync_settings = SyncSettings::new().timeout(Duration::from_millis(3000));

        let response = client.sync_once(sync_settings).await.unwrap();

        assert_ne!(response.next_batch, "");

        assert!(client.sync_token().await.is_some());
    }

    #[tokio::test]
    async fn test_autojoin() {
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
