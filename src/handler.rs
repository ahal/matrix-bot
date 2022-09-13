use matrix_sdk::{
    async_trait,
    room::Joined,
};

use crate::MatrixBot;

pub enum HandleResult {
    Continue,
    Stop,
}

#[async_trait]
pub trait MessageHandler {
    async fn handle_message(
        &self,
        bot: &MatrixBot,
        room: &Joined,
        msg: &str,
    ) -> HandleResult;
}
