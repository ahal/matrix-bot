use matrix_bot::{
    MatrixBot,
    handler::{HandleResult, MessageHandler},
};
use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    room::Room,
};
use uuid::Uuid;

pub struct UuidHandler {

}

impl UuidHandler {
    fn generate_uuid(&self) -> std::string::String {
        Uuid::new_v4().to_hyphenated().to_string()
    }
}

#[async_trait]
impl MessageHandler for UuidHandler {
    async fn handle_message(&self, _bot: &MatrixBot, room: &Room, event: &SyncMessageEvent<MessageEventContent>) -> HandleResult {
        if let Room::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                    ..
                },
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };

            if msg_body == "!uuid" {
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(
                    self.generate_uuid()
                ));

                println!("sending");

                room.send(content, None).await.unwrap();

                println!("message sent");
                return HandleResult::Stop
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
