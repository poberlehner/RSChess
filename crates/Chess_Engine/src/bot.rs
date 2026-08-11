use rand::rngs;

use crate::board::Board;
use crate::board::r#move::Move;

pub trait ChessBot {
    fn get_best_move(&mut self, board: &mut Board, rng: &mut rngs::StdRng) -> Move;
}