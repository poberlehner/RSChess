use chess_engine::board::Board;
use chess_engine::board::r#move::Move;

pub trait ChessBot {
    fn get_best_move(&mut self, board: &mut Board) -> Move;
}