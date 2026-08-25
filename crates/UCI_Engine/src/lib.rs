use std::io::{self, Write, stdin, stdout};
use std::str::FromStr;
use std::thread::JoinHandle;
use chess_engine::board::{Board, STARTFEN, r#move::Move, utilities::TimeControl};
use chess_bot::bot::Bot;
use log::warn;

#[allow(unused)]
pub struct UCIController {
    board: Board,
    bot: Bot,

    ///Only for the bot as the bot_thread keeps it locked for it's entire runtime
    bot_thread: Option<JoinHandle<Move>>,
}

impl UCIController {
    pub fn new() -> UCIController {
        UCIController {
            board: Board::new(),
            bot: Bot::new(),

            bot_thread: None,
        }
    }

    pub fn run(&mut self) {
        println!("UCI engine RSChess is running");
        let mut command = String::new();
        let mut result: Result<usize, io::Error>;


        while command != "quit" {
            print!("> ");
            let r = stdout().flush();
            if let Err(e) = r {
                println!("Error while flushing stdout: {e}");
            }

            command.clear();
            result = stdin().read_line(&mut command);
            command.pop();

            if let Err(e) = result {
                eprintln!("IO Error encountered: {}", e);
                return;
            }

            let cmd_copy = command.clone();
            let mut commands = cmd_copy.split_whitespace().collect::<Vec<&str>>();

            match commands[0] {
                "uci" => { println!("uciok"); }
                "isready" => { println!("isreadyok"); }
                "ucinewgame" => {
                    self.board.load_fen(STARTFEN);
                    self.board.start_game(TimeControl::Unlimited);
                }
                "position" => {
                    self.process_position_cmd(&mut commands);
                }
                "go" => {
                    self.process_go_cmd(&mut commands);
                }
                "stop" => {
                    eprintln!("RSChess cannot stop its search early yet");
                    warn!("RSChess cannot stop its search early yet");
                }
                "d" => {
                    println!("{}", self.board.generate_fen());
                }
                "quit" => {}
                _ => { println!("not an implemented command"); }
            }
        }
    }

    fn process_position_cmd(&mut self, commands: &mut [&str]) {
        //TODO: Test
        let mut move_index = 2;
        if commands[1] == "fen" {
            if commands[2] == "startpos" {
                self.board.load_fen(STARTFEN);
            }
            else {
                let mut fen = String::new();
                while (commands[move_index] != "moves") && move_index < commands.len() {
                    fen.push_str(commands[move_index]);
                    fen.push(' ');
                    move_index += 1;
                }

                fen.pop();

                self.board.load_fen(&fen);
            }
        }

        move_index += 1;
        self.board.start_game(TimeControl::Unlimited);

        while move_index < commands.len() {

            self.board.make_move(Move::from_str(commands[move_index]).expect(""), false);
            move_index += 1;
        }
    }

    #[allow(unused)]
    fn process_go_cmd(&mut self, commands: &mut [&str]) {
        todo!();

        match commands[1] {
            "infinite" => {
                
            }
            "depth" => {
                
            }
            _ => {}
        }
    }
}

impl Default for UCIController {
    fn default() -> Self {
        Self::new()
    }
}