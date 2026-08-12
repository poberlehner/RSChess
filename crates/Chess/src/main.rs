use bot_controller::bot_controller::BotController;
use uci_engine::UCIController;

fn main() {
    let mut arguments: Vec<String> = std::env::args().collect();
    arguments.remove(0);

    if arguments.len() >= 1 {
        match arguments.remove(0).as_str() {
            "gui" => {
                let mut app: BotController = BotController::new();

                if arguments.len() == 0 {
                    app.run_bot_vs_bot();

                    return;
                }

                match arguments.remove(0).as_str() {
                    "botvbot" => {
                        app.run_bot_vs_bot();
                    }
                    "botvhuman" => {
                        app.run_human_vs_bot();
                    }
                    "humanvhuman" => {
                        app.run_human_vs_human();
                    }
                    _ => { println!("Invalid argument after gui "); }
                }
            }
            "uci" => {
                let mut uci = UCIController::new();
                uci.run();
            }
            _ => { println!("Invalid argument"); }
        }
    }
    else {
        let mut uci = UCIController::new();
        uci.run();
    }
}

//TODO: Replace is_some with bitboard checks
//TODO: Implement piece lists in the board struct to allow for faster evaluation