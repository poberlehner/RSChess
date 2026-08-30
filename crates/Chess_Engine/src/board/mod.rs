mod initialization;
pub mod utilities;
pub mod gamestate;
pub mod r#move;
mod movegeneration;
mod bitboards;
mod generalapi;
pub mod piece;
mod tests;
pub mod zobrist;
mod precomputation;

use chrono::{DateTime, Local};
use gamestate::GameState;
use piece::Piece;
use r#move::Move;
use crate::board::gamestate::GameResult;
use crate::board::r#move::LogMove;
use crate::board::utilities::TimeControl;
use zobrist::Zobrist;

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pub(super) squares: [Option<Piece>; 64],

    pub(super) game_state: GameState,
    game_result: GameResult,

    time_control: TimeControl,
    move_start_time: std::time::Instant,
    white_time_left: std::time::Duration,
    black_time_left: std::time::Duration,

    zobrist_history: Vec<i64>,
    zobrist: Zobrist,

    white_king_pos: u8,
    black_king_pos: u8,
    check: bool,
    white_double_check: Option<bool>,
    black_double_check: Option<bool>,

    distance_to_edge: [[u8; 8]; 64],
    knight_jump_legality: [[bool; 8]; 64],

    pub(super) legal_moves: Option<Vec<Move>>,

    move_log: Vec<LogMove>,
    start_fen: String,
    game_start_time: DateTime<Local>,

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

pub const STARTFEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";