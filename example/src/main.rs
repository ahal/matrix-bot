use matrix_bot::{
    MatrixBot,
    apps::uuid::UuidHandler,
};

fn main() {
    let mut bot = MatrixBot::new().unwrap();
    bot.add_handler(UuidHandler {});
    bot.run().unwrap();
}
