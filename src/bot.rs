use crate::handler::{HandleResult, MessageHandler};
use matrix_sdk::{room::Room, ruma::events::room::message::OriginalSyncRoomMessageEvent};

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
        M: handler::MessageHandler + Send + Sync + 'static,
    {
        self.handlers.push(handler);
    }

    pub async fn on_room_message(&self, room: &Room, event: &OriginalSyncRoomMessageEvent) {
        for handler in self.handlers.iter() {
            let val = handler.handle_message(room, event).await;
            match val {
                HandleResult::Continue => continue,
                HandleResult::Stop => break,
            }
        }
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    use matrix_sdk::{identifiers::room_id, Client};
//    use matrix_sdk_test::{test_json, EventBuilder};
//    use mockito::mock;
//    use serde_json::json;
//    use std::fs::File;
//    use std::io::Write;
//    use tempfile::tempdir;
//    use url::form_urlencoded::byte_serialize;
//
//    async fn get_client() -> Client {
//        let homeserver = mockito::server_url();
//        let _m = mock("POST", "/_matrix/client/r0/login")
//            .with_status(200)
//            .with_body(test_json::LOGIN.to_string())
//            .create();
//
//        let config = &format!(
//            r#"
//            homeserver = "{}"
//            username = "user"
//            password = "password"
//            statedir = "{}"
//        "#,
//            homeserver,
//            tempdir().unwrap().path().to_str().unwrap()
//        );
//
//        let dir = tempdir().unwrap();
//        let path = dir.path().join("config.toml");
//        let mut file = File::create(&path).unwrap();
//        writeln!(file, "{}", config).unwrap();
//
//        let bot = MatrixBot::new(&path.to_str().unwrap()).await.unwrap();
//        let client = bot.client.clone();
//        client.set_event_handler(Box::new(bot)).await;
//        client
//    }
//
//    #[tokio::test]
//    async fn login() {
//        let client = get_client().await;
//        let logged_in = client.logged_in().await;
//        assert!(logged_in, "Bot should be logged in");
//    }
//
//    #[tokio::test]
//    async fn autojoin() {
//        let client = get_client().await;
//        let room_id = room_id!("!SVkFJHzfwvuaIEawgC:localhost");
//        let room = client.get_invited_room(&room_id);
//        assert!(room.is_none());
//
//        let event = json!({
//            "content": {
//                "displayname": "example",
//                "membership": "join"
//            },
//            "event_id": "$151800140517rfvjc:localhost",
//            "membership": "join",
//            "origin_server_ts": 0,
//            "sender": "@example:localhost",
//            "state_key": "@cheeky_monkey:matrix.org",
//            "type": "m.room.member"
//        });
//
//        let sync_response_json = EventBuilder::default()
//            .add_custom_invited_event(&room_id, event)
//            .build_json_sync_response();
//
//        let m1 = mock(
//            "GET",
//            mockito::Matcher::Regex(r"^/_matrix/client/r0/sync.*".to_string()),
//        )
//        .with_status(200)
//        .with_body(sync_response_json.to_string())
//        .create();
//
//        let encoded_room_id: String = byte_serialize(room_id.as_bytes()).collect();
//        let url = format!("/_matrix/client/r0/rooms/{}/join", encoded_room_id);
//        let m2 = mock("POST", url.as_str())
//            .with_status(200)
//            .with_body(json!({ "room_id": &room_id }).to_string())
//            .create();
//
//        client.sync_once(SyncSettings::default()).await.unwrap();
//        m1.assert();
//        m2.assert();
//    }
//}
