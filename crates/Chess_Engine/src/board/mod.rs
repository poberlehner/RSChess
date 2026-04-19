mod initialization;
pub mod utilities;
pub mod gamestate;
pub mod r#move;
mod movegeneration;
mod bitboards;
mod generalapi;
pub mod piece;
mod tests;
mod zobrist;

use gamestate::GameState;
use piece::Piece;
use r#move::Move;
use crate::board::gamestate::GameResult;
use crate::board::r#move::LogMove;
use crate::board::utilities::TimeControl;
use zobrist::Zobrist;

#[derive(Clone, PartialEq)]
pub struct Board {
    pub(super) squares: [[Option<Piece>; 8]; 8],

    pub(super) game_state: GameState,
    game_result: GameResult,
    //Limit game end conditions to checkmate for the perft function
    full_game_end_enforcement: bool,

    time_control: TimeControl,
    move_start_time: std::time::Instant,
    white_used_time: std::time::Duration,
    black_used_time: std::time::Duration,

    zobrist_history: Vec<i64>,
    zobrist: Zobrist,

    white_king_pos: (u8, u8),
    black_king_pos: (u8, u8),
    check: bool,
    white_double_check: Option<bool>,
    black_double_check: Option<bool>,

    pub(super) legal_moves: Option<Vec<Move>>,

    move_log: Vec<LogMove>,

    white_pieces_bitboard: Option<i64>,
    black_pieces_bitboard: Option<i64>,
    white_attacked_squares_bitboard: Option<i64>,
    black_attacked_squares_bitboard: Option<i64>,
    white_orthogonal_pin_bitboard: Option<i64>,
    white_diagonal_pin_bitboard: Option<i64>,
    black_orthogonal_pin_bitboard: Option<i64>,
    black_diagonal_pin_bitboard: Option<i64>,
    white_check_block_bitboard: Option<i64>,
    black_check_block_bitboard: Option<i64>,

    white_en_passant_pin_bitboard: Option<i64>,
    black_en_passant_pin_bitboard: Option<i64>,
}