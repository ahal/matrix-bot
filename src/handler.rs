use crate::MatrixBot;

use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::MessageEventContent,
        SyncMessageEvent,
    },
    room::Room,
};

/// Possible return values of `MessageHandler.handle_message`. `HandleResult::Continue` if the
/// message should be passed on to the next handler. `HandleResult::Stop` if it should not.
pub enum HandleResult {
    Continue,
    Stop,
}

/// Trait for handling messages.
///
/// # Example
///
/// ```
/// pub struct EchoHandler {}
///
/// #[async_trait]
/// impl MessageHandler for EchoHandler {
///     async fn handle_message(&self, _bot: &MatrixBot, room: &Room, event: &SyncMessageEvent<MessageEventContent>) -> HandleResult {
///         if let Room::Joined(room) = room {
///             let msg_body = if let SyncMessageEvent {
///                 content: MessageEventContent {
///                     msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
///                     ..
///                 },
///                 ..
///             } = event
///             {
///                 msg_body.clone()
///             } else {
///                 String::new()
///             };
///
///             room.send(msg_body, None).await.unwrap();
///             return HandleResult::Stop
///         }
///         HandleResult::Continue
///     }
/// }
/// ```
#[async_trait]
pub trait MessageHandler {
    async fn handle_message(
        &self,
        bot: &MatrixBot,
        room: &Room,
        event: &SyncMessageEvent<MessageEventContent>,
    ) -> HandleResult;
}
