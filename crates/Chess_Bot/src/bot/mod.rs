#![allow(dead_code)]

use chess_engine::board::r#move::Move;
use chess_engine::bot::ChessBot;
use chess_engine::board::Board;

pub struct Bot {
    piece_values: [i32; 6]
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            piece_values: [100, 300, 300, 500, 900, 999999]
        }
    }
}

impl ChessBot for Bot {
    fn play_move(&mut self, board: &mut Board) -> Move {
        let moves = board.get_legal_moves().clone().unwrap();

        board.make_move(moves[0], false);

        moves[0]
    }
}