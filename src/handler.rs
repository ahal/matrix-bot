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
