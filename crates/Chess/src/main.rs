use std::time::*;
use bot_controller::bot_controller::BotController;

fn main() {
    let start_time = Instant::now();
    let mut app: BotController = BotController::new();
    println!("Starting time:  {:?}", start_time.elapsed());

    app.run_bot_vs_bot();
}

//TODO: Implement the get_move_history function
//TDOO: Fix the fifty move rule kicking in too early
//TODO: Replace is_some with bitboard checks
//TODO: Implement piece lists in the board struct to allow for faster evaluation
//TODO: Use multithreading to prevent lag while the Bot is calculating