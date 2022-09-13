use matrix_sdk::{
    async_trait,
    room::Room,
    ruma::events::room::message::OriginalSyncRoomMessageEvent
};

pub enum HandleResult {
    Continue,
    Stop,
}

#[async_trait]
pub trait MessageHandler {
    async fn handle_message(
        &self,
        room: &Room,
        event: &OriginalSyncRoomMessageEvent,
    ) -> HandleResult;
}
