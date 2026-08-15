use log::*;
use chess_engine::board::{r#move::Move, piece::PieceType};
use chess_engine::bot::ChessBot;
use chess_engine::board::Board;

#[derive(Clone, Copy)]
pub struct Bot {
    node_count: i64,
    best_move: Move,
}

//The tables are vertically inverted compared to the default gui view
static PIECE_SQUARE_TABLES: [[i64; 64]; 6] = [
    [    0,   0,   0,   0,   0,   0,   0,   0,
		 5,  10,  10, -20, -20,  10,  10,   5,
		 5,  -5, -10,   0,   0, -10,  -5,   5,
		 0,   0,   0,  20,  20,   0,   0,   0,
		 5,   5,  10,  25,  25,  10,   5,   5,
		10,  10,  20,  30,  30,  20,  10,  10,
		50,  50,  50,  50,  50,  50,  50,  50,
		 0,   0,   0,   0,   0,   0,   0,   0   ], //Pawn
    [	-50,-40,-30,-30,-30,-30,-40,-50,
		-40,-20,  0,  5,  5,  0,-20,-40,
		-30,  5, 10, 15, 15, 10,  5,-30,
		-30,  0, 15, 20, 20, 15,  0,-30,
		-30,  5, 15, 20, 20, 15,  5,-30,
		-30,  0, 10, 15, 15, 10,  0,-30,
		-40,-20,  0,  0,  0,  0,-20,-40,
		-50,-40,-30,-30,-30,-30,-40,-50,  ], //Knight
    [	-20,-10,-10,-10,-10,-10,-10,-20,
		-10,  5,  0,  0,  0,  0,  5,-10,
		-10, 10, 10, 10, 10, 10, 10,-10,
		-10,  0, 10, 10, 10, 10,  0,-10,
		-10,  5,  5, 10, 10,  5,  5,-10,
		-10,  0,  5, 10, 10,  5,  0,-10,
		-10,  0,  0,  0,  0,  0,  0,-10,
		-20,-10,-10,-10,-10,-10,-10,-20   ], //Bishop
    [	 0,  0,  0,  5,  5,  0,  0,  0,
		-5,  0,  0,  0,  0,  0,  0, -5,
		-5,  0,  0,  0,  0,  0,  0, -5,
		-5,  0,  0,  0,  0,  0,  0, -5,
		-5,  0,  0,  0,  0,  0,  0, -5,
		-5,  0,  0,  0,  0,  0,  0, -5,
		 5, 10, 10, 10, 10, 10, 10,  5,
      	 0,  0,  0,  0,  0,  0,  0,  0,   ], //Rook
    [   -20, -10,-10, -5, -5,-10,-10,-20,
		-10,   0,  5,  0,  0,  0,  0,-10,
		-10,   5,  5,  5,  5,  5,  0,-10,
		  0,   0,  5,  5,  5,  5,  0, -5,
		 -5,   0,  5,  5,  5,  5,  0, -5,
		-10,   0,  5,  5,  5,  5,  0,-10,
		-10,   0,  0,  0,  0,  0,  0,-10,
		-20, -10,-10, -5, -5,-10,-10,-20  ], //Queen
    [	 20,  30,  10,   0,   0,  10,  30,  20,
		 20,  20,  -5,  -5,  -5,  -5,  20,  20,
		-10, -20, -20, -20, -20, -20, -20, -10,
		-20, -30, -30, -40, -40, -30, -30, -20,
		-30, -40, -40, -50, -50, -40, -40, -30,
		-40, -50, -50, -60, -60, -50, -50, -40,
		-60, -60, -60, -60, -60, -60, -60, -60,
		-80, -70, -70, -70, -70, -70, -70, -80, ] //King
];

static DEPTH: i32 = 5;
static PIECE_VALUES: [i64; 6] = [100, 300, 300, 500, 900, 100000];

impl Bot {
    pub fn new() -> Bot {
        Bot {
            node_count: 0,
            best_move: Move { origin_square: 0, destination_square: 0, promotion_type: PieceType::None },
        }
    }
}

impl ChessBot for Bot {
    fn get_best_move(&mut self, board: &mut Board) -> Move {
        self.search(board, DEPTH, -99999999, 99999999);

        println!("Node count: {}", self.node_count);
        self.node_count = 0;

        self.best_move
    }
}

impl Bot {
    fn search(&mut self, board: &mut Board, depth: i32, alpha: i64, beta: i64) -> i64 {
        if DEPTH != depth {
            match board.get_game_result_simplified() {
                0 => { return 0; }
                1 => { return (DEPTH - depth) as i64 - 9999999; }
                2 => { return (DEPTH - depth) as i64 - 9999999; }
                _ => {}
            }
        }

        if depth == 0 {
            return self.evaluate(board);
        }

        let moves = board.get_legal_moves().clone().unwrap();
        let mut best_value = alpha;
        let mut best_move: Move = moves[0];

        for m in moves {
            let mut value: i64 = 0;
            if !board.make_move(m, true) {
                error!("Failed to make move {} in position {}", m.to_string(), board.generate_fen());
            }
            else {
                value = -self.search(board, depth - 1, -beta, -best_value);
            }

            board.undo_move(true).expect("Failed to undo move");

            if value > best_value {
                best_value = value;
                best_move = m;

                if DEPTH != depth {
                    if value >= beta {
                        return beta;
                    }
                }
            }
        }

        if DEPTH == depth {
            println!("Best move: {}, value: {best_value}", self.best_move.to_string());
        }

        self.best_move = best_move;

        best_value
    }

    //Returns the eval from the perspective of the side to move
    pub fn evaluate(&mut self, board: &mut Board) -> i64 {
        let squares = board.get_squares();
        let mut white_value: i64 = 0;
        let mut black_value: i64 = 0;

        self.node_count += 1;

        for i in 0..64 {
            if squares[i].is_none() {
                continue;
            }

            let p = squares[i].unwrap();
            let i_corr = if board.get_white_to_move() { i } else { 64 - i - 1 };

            let piece_val = PIECE_SQUARE_TABLES[p.piece_type as usize - 1][i_corr] + PIECE_VALUES[p.piece_type as usize - 1];

            if p.white_piece { white_value += piece_val; }
            else { black_value += piece_val; }
        }

        let mut value = white_value - black_value;

        if !board.get_white_to_move() {
            value *= -1;
        }

        value
    }
}