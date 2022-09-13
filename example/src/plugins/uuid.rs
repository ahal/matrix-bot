use matrix_bot::{handler::{HandleResult, MessageHandler}, MatrixBot};
use matrix_sdk::{
    self, async_trait,
    room::Joined,
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
        bot: &MatrixBot,
        room: &Joined,
        msg: &str,
    ) -> HandleResult {
        if msg == "!uuid" {
            println!("sending");
            bot.send(room, self.generate_uuid()).await.unwrap();
            println!("message sent");
            return HandleResult::Stop;
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
