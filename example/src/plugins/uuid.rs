use matrix_bot::{
    handler::{HandleResult, MessageHandler},
    MatrixBot,
};
use matrix_sdk::{
    self, async_trait,
    room::Room,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
    },
};
use uuid::Uuid;

pub struct UuidHandler {}

impl UuidHandler {
    fn generate_uuid(&self) -> std::string::String {
        Uuid::new_v4().to_hyphenated().to_string()
    }
}

#[async_trait]
impl MessageHandler for UuidHandler {
    async fn handle_message(
        &self,
        room: &Room,
        event: &OriginalSyncRoomMessageEvent,
    ) -> HandleResult {
        if let Room::Joined(room) = room {
            let msg_body = match &event.content.msgtype {
                MessageType::Text(TextMessageEventContent { body, .. }) => body,
                _ => return HandleResult::Stop,
            };

            if msg_body == "!uuid" {
                let content = RoomMessageEventContent::text_plain(self.generate_uuid());

                println!("sending");

                room.send(content, None).await.unwrap();

                println!("message sent");
                return HandleResult::Stop;
            }
        }
        HandleResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_is_generated() {
        let handler = UuidHandler {};
        let uuid = handler.generate_uuid();
        assert_eq!(uuid.len(), 36);
    }
}
