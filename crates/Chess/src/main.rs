mod logtarget;

use bot_controller::bot_controller::BotController;
use env_logger::Env;
use log::*;
use uci_engine::UCIController;

fn main() {
    let mut log_target = Box::new(logtarget::LogTarget::new("logs/Chess.log".to_string(),
                                    LevelFilter::Info, LevelFilter::Off));

    let mut arguments: Vec<String> = std::env::args().collect();
    arguments.remove(0);

    if arguments.is_empty() {
        env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_target))).parse_env(Env::default().default_filter_or("trace")).init();
        let mut uci = UCIController::new();
        uci.run();
    }
    else {
        match arguments[0].as_str() {
            "gui" => {
                log_target.console_log_level = LevelFilter::Trace;
                env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_target))).parse_env(Env::default().default_filter_or("trace")).init();

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
                        error!("Invalid argument after \"Chess gui \": {}", arguments[1]);
                    }
                }
            }
            "uci" => {
                env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_target))).parse_env(Env::default().default_filter_or("trace")).init();
                let mut uci = UCIController::new();
                uci.run();
            }
            _ => {
                env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_target))).parse_env(Env::default().default_filter_or("trace")).init();
                error!("Invalid command line argument: \"{}\". Valid arguments are: gui, uci", arguments[0]);
            }
        }
    }
}

//TODO: Replace is_some with bitboard checks
//TODO: Implement piece lists in the board struct to allow for faster evaluation