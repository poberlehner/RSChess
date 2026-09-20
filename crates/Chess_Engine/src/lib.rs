#![allow(clippy::collapsible_if)]
pub mod board;
pub mod renderer;

#[cfg(test)]
mod tests {
    use crate::board::{Board, utilities::TimeControl};
    use std::env;
    use std::fs::{exists, File};
    use std::io::Write;
    use std::path::PathBuf;
    use env_logger;
    #[allow(unused)]
    use log::*;
    use crate::board::r#move::{LogMove, Move};

    #[test]
    pub fn test_move_generation() {
        let current_directory = env::current_dir();
        let mut current_directory = current_directory.unwrap();

        while !(exists(current_directory.join(PathBuf::from("assets"))).is_ok() && exists(current_directory.join(PathBuf::from("assets"))).unwrap())  {
            current_directory.pop();
        }

        env::set_current_dir(current_directory).expect("Failed to set current working directory");


        let log_file = File::options().append(true).create(true).write(true).open("assets/Chess.log").expect("Failed to open log file");
        env_logger::Builder::new().target(env_logger::Target::Pipe(Box::new(log_file))).filter_level(log::LevelFilter::Trace).init();

        let mut board: Board = Board::new();

        let num_positions: u64;

        board.start_game(TimeControl::Unlimited);

        let mut executed_moves_log: Vec<(Move, u32)> = vec![];
        executed_moves_log.reserve(5999999);
        let mut undone_moves_log: Vec<(LogMove, u32)> = vec![];
        undone_moves_log.reserve(5999999);

        static DEPTH: u32 = 6;

        num_positions = recursive_move_generation_test(&mut board, DEPTH, DEPTH, &mut executed_moves_log, &mut undone_moves_log);

        println!("Number of positions encountered: {}", num_positions);

        std::io::stdout().flush().expect("Failed to flush stdout");
    }

    fn recursive_move_generation_test
        (board: &mut Board, depth: u32, max_depth: u32, move_log: &mut Vec<(Move, u32)>, undo_move_log: &mut Vec<(LogMove, u32)>) -> u64
    {
        let legal_moves = board.get_legal_moves().clone().expect("Cloning the legal move vector failed");

        if depth == 1 {
            return legal_moves.len() as u64;
        }

        let mut num_positions = 0;

        // board.test_duplicate_moves();

        for legal_move in legal_moves {
            // let mut board_clone = board.clone();
            // let fen_before = board.generate_fen();

            // move_log.push((legal_move, max_depth - depth));
            if !board.make_move(legal_move, true) {
                // if !undo_move_log.is_empty() {
                //     eprintln!("Last undone move: {:?}", undo_move_log[undo_move_log.len() - 1]);
                //     warn!("Make move failed after undoing move {:?}", undo_move_log[undo_move_log.len() - 1]);
                // }
                // eprintln!("Executing move {} failed (depth: {})", legal_move.to_string(), max_depth - depth);
            }

            let num_new_positions = recursive_move_generation_test(board, depth - 1, max_depth, move_log, undo_move_log);
            num_positions += num_new_positions;

            if depth == max_depth {
                eprintln!("{}: {}", legal_move.to_string(), num_new_positions);
            }

            // undo_move_log.push((board.undo_move(true).unwrap(), max_depth - depth));
            board.undo_move(true);

            // board.invalidate_caches();
            // board_clone.invalidate_caches();
            // if board_clone != *board {
            //     println!("Board undo incomplete");

            //     if board_clone.game_state != (*board).game_state {
            //         println!("{:?}", undo_move_log[undo_move_log.len() - 1]);
            //         println!("Game State error while undoing a move");
            //     }
            //     if board_clone.squares != (*board).squares {
            //         println!("Fen before: {}", fen_before);
            //         println!("{:?}", undo_move_log[undo_move_log.len() - 1]);
            //         println!("Pieces incorrectly moved while undoing a move");

            //         for i in 0..8 {
            //             if board_clone.squares[i] != (*board).squares[i] {
            //                 println!("Pieces incorrectly moved, discrepancy in column {}", i);
            //             }
            //         }
            //     }
            // }
        }

        num_positions
    }
}
