use matrix_bot::{
    MatrixBot,
    plugins::uuid::UuidHandler,
};

fn main() {
    let mut bot = MatrixBot::new().unwrap();
    bot.add_handler(UuidHandler {});
    bot.run().unwrap();
}
