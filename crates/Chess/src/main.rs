use std::{fs::File, io::Write};
use bot_controller::bot_controller::BotController;
use env_logger::Env;
use log::*;
use uci_engine::UCIController;

fn main() {
    let mut log_file = File::options().append(true).create(true).write(true).open("assets/Chess.log").expect("Failed to open log file");
    if let Err(e) = log_file.write("\n".as_bytes()) {
        eprintln!("Failed to write to log file \"assets/Chess.log\" with error {e}");
    }
    env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_file))).parse_env(Env::default().default_filter_or("info")).init();

    let mut arguments: Vec<String> = std::env::args().collect();
    arguments.remove(0);

    if arguments.len() >= 1 {
        match arguments[0].as_str() {
            "gui" => {
                let mut app: BotController = BotController::new();

                if arguments.len() == 1 {
                    info!("Running default gui bot vs. bot");
                    app.run_bot_vs_bot();

                    return;
                }

                match arguments[1].as_str() {
                    "botvbot" => {
                        app.run_bot_vs_bot();
                    }
                    "botvhuman" => {
                        app.run_human_vs_bot();
                    }
                    "humanvhuman" => {
                        app.run_human_vs_human();
                    }
                    _ => {
                        println!("Invalid argument after gui ");
                        error!("Invalid argument after \"Chess gui \": {}", arguments[1]);
                    }
                }
            }
            "uci" => {
                let mut uci = UCIController::new();
                uci.run();
            }
            _ => {
                println!("Invalid argument\nValid arguments are: gui, uci");
                error!("Invalid command line argument: {}", arguments[0]);
            }
        }
    }
    else {
        let mut uci = UCIController::new();
        uci.run();
    }
}

//TODO: Replace is_some with bitboard checks
//TODO: Implement piece lists in the board struct to allow for faster evaluation