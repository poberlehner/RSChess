use rand::{RngExt, rngs::SmallRng};

use crate::board::Board;

#[derive(PartialEq, Clone, Copy)]
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

        for square_index in 0..64 {
            for piece_index in 0..15 {
                zobrist.square_nums[square_index][piece_index] = rng.random::<i64>();
            }
        }

        for castling_state_index in 0..16 {
            zobrist.castling_states_nums[castling_state_index] = rng.random::<i64>();
        }

        for en_passant_index in 0..9 {
            zobrist.en_passant[en_passant_index] = rng.random::<i64>();
        }

        zobrist.side_to_move = rng.random::<i64>();

        #[cfg(debug_assertions)]
        println!("Zobrist initialization time: {}µs", start_time.elapsed().as_micros());

        zobrist
    }

    pub fn generate_key(&self, board: &Board) -> i64 {
        let mut key: i64 = 0;

        for i in 0..64 {
            let piece = board.squares[i % 8][i / 8];

            if piece.is_some() {
                key ^= self.square_nums[i][piece.unwrap().as_num()];
            }
        }

        if board.game_state.en_passant_square.is_none() {
            key ^= self.en_passant[0];
        }
        else {
            key ^= self.en_passant[(board.game_state.en_passant_square.unwrap().0 + 1) as usize];
        }

        if board.game_state.white_to_move {
            key ^= self.side_to_move;
        }

        let castle = board.game_state.castling_rights;
        key ^= self.castling_states_nums[castle[0] as usize + 2 * castle[1] as usize + 4 * castle[2] as usize + 8 * castle[3] as usize];

        key
    }
}