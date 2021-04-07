use matrix_bot::MatrixBot;

pub mod plugins {
    pub mod uuid;
}
use crate::plugins::uuid::UuidHandler;

fn main() {
    let mut bot = MatrixBot::new().unwrap();
    bot.add_handler(UuidHandler {});
    bot.run().unwrap();
}
