use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    SyncRoom,
};
use uuid::Uuid;

use crate::MatrixBot;
use crate::handler::{HandleResult, MessageHandler};

pub struct UuidHandler {}

#[async_trait]
impl MessageHandler for UuidHandler {
    async fn handle_message(&self, bot: &MatrixBot, room: &SyncRoom, event: &SyncMessageEvent<MessageEventContent>) -> HandleResult {
        if let SyncRoom::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };

            if msg_body == "!uuid" {
                let new_uuid = Uuid::new_v4();
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(
                    new_uuid.to_hyphenated().to_string(),
                ));

                println!("sending");

                // we clone here to hold the lock for as little time as possible.
                let room_id = room.read().await.room_id.clone();
                bot.client
                    // send our message to the room we found the "!party" command in
                    // the last parameter is an optional Uuid which we don't care about.
                    .room_send(&room_id, content, None)
                    .await
                    .unwrap();

                println!("message sent");
                return HandleResult::Stop
            }
        }
        HandleResult::Continue
    }
}
