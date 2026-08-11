use std::time::*;
use bot_controller::bot_controller::BotController;

fn main() {
    let start_time = Instant::now();
    let mut app: BotController = BotController::new();
    println!("Starting time:  {:?}", start_time.elapsed());

    app.run_bot_vs_bot();
}

//TODO: Add a UCIController bin crate to allow for easier Testing
//TODO: Replace is_some with bitboard checks
//TODO: Implement piece lists in the board struct to allow for faster evaluation