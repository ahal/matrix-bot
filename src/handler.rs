use crate::MatrixBot;

use matrix_sdk::{
    self, async_trait,
    events::{
        room::{
            message::MessageEventContent,
        },
        SyncMessageEvent,
    },
    SyncRoom,
};

pub enum HandleResult {
    Continue,
    Stop,
}

#[async_trait]
pub trait MessageHandler {
    async fn handle_message(
        &self,
        bot: &MatrixBot,
        room: &SyncRoom,
        event: &SyncMessageEvent<MessageEventContent>,
    ) -> HandleResult;
}

pub fn extract_command<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    if message.starts_with(prefix) {
        let new_start = prefix.len();
        let key = message[new_start..].split_whitespace().next().unwrap_or("");
        return Some(&key);
    }
    None
}
