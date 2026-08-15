use crate::board::gamestate::{GameResult, GameState};
use crate::board::Board;
use crate::board::utilities::TimeControl::Unlimited;
use crate::board::zobrist::Zobrist;
use chrono::Local;
use log::*;
use std::time::{Duration, Instant};

impl Board {
    pub fn new() -> Board {
        let total_start_time = Instant::now();
        let mut start_time = Instant::now();

        let mut board = Board {
            squares: [None; 64],

            game_result: GameResult::NotStarted,
            game_state: GameState {
                white_to_move: true,
                castling_rights: [false; 4],
                en_passant_square: None,
                fifty_move_counter: 0,
                next_move_number: 1,
            },

            time_control: Unlimited,
            white_time_left: Duration::new(600, 0),
            black_time_left: Duration::new(600, 0),
            move_start_time: Instant::now(),

            zobrist_history: vec![],
            zobrist: Zobrist::new(),

            legal_moves: None,

            white_king_pos: 0,
            black_king_pos: 0,
            check: false,
            white_double_check: None,
            black_double_check: None,

            distance_to_edge: [[0; 8]; 64],
            knight_jump_legality: [[false; 8]; 64],

            move_log: vec![],
            start_fen: String::from(""),
            game_start_time: Local::now(),

            white_attacked_squares_bitboard: None,
            black_attacked_squares_bitboard: None,
            white_orthogonal_pin_bitboard: None,
            black_orthogonal_pin_bitboard: None,
            white_en_passant_pin_bitboard: None,
            black_en_passant_pin_bitboard: None,
            white_diagonal_pin_bitboard: None,
            black_diagonal_pin_bitboard: None,
            white_check_block_bitboard: None,
            black_check_block_bitboard: None,
            white_pieces_bitboard: None,
            black_pieces_bitboard: None,
        };

        trace!("board creation took {}µs", start_time.elapsed().as_micros());

        board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        start_time = Instant::now();

        board.move_log.reserve(40);

        board.precompute_distance_to_edge_data();
        board.precompute_knight_jump_legality();

        trace!("Precomputation took {}µs", start_time.elapsed().as_micros());
        start_time = Instant::now();

        board.generate_all_bitboards();
        board.generate_pieces_bitboard(true);
        board.generate_pieces_bitboard(false);

        trace!("Bitboard generation took {}µs", start_time.elapsed().as_micros());
        info!("Board init took {}µs", total_start_time.elapsed().as_micros());

        board
    }
}