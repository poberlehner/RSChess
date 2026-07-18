use crate::board::Board;
use crate::board::r#move::Move;

pub trait ChessBot {
    fn play_move(&mut self, board: &mut Board) -> Move;
}