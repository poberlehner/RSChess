#[cfg(debug_assertions)]
use log::trace;
use rand::{RngExt, rngs::SmallRng};

use crate::board::Board;

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Zobrist {
    pub square_nums: [[i64; 15]; 64],

    pub castling_states_nums: [i64; 16],

    pub en_passant: [i64; 9],
    pub side_to_move: i64
}

impl Zobrist {
    pub fn new() -> Zobrist {
        #[cfg(debug_assertions)]
        let start_time = std::time::Instant::now();

        let mut zobrist: Zobrist = Zobrist { square_nums: [[0; 15]; 64], castling_states_nums: [0; 16], en_passant: [0; 9], side_to_move: 0 };
        let mut rng: SmallRng = rand::make_rng();

        for piece_nums in &mut zobrist.square_nums {
            for num in piece_nums.iter_mut() {
                *num = rng.random::<i64>();
            }
        }

        for castling_state in &mut zobrist.castling_states_nums {
            *castling_state = rng.random::<i64>();
        }

        for en_passant_file in &mut zobrist.en_passant {
            *en_passant_file = rng.random::<i64>();
        }

        zobrist.side_to_move = rng.random::<i64>();

        #[cfg(debug_assertions)]
        trace!("Zobrist initialization time: {}µs", start_time.elapsed().as_micros());

        zobrist
    }

    pub fn generate_key(&self, board: &Board) -> i64 {
        let mut key: i64 = 0;

        for i in 0..64 {
            let piece = board.squares[i];

            if let Some(piece) = piece {
                key ^= self.square_nums[i][piece.as_num()];
            }
        }

        if let Some(en_passant_square) = board.game_state.en_passant_square {
            key ^= self.en_passant[(en_passant_square % 8 + 1) as usize];
        }
        else {
            key ^= self.en_passant[0];
        }

        if board.game_state.white_to_move {
            key ^= self.side_to_move;
        }

        let castle = board.game_state.castling_rights;
        key ^= self.castling_states_nums[castle[0] as usize + 2 * castle[1] as usize + 4 * castle[2] as usize + 8 * castle[3] as usize];

        key
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}