use std::panic;
use chess_engine::board::r#move::Move;
use chess_engine::bot::ChessBot;
use chess_engine::board::Board;
use rand::{RngExt, rngs::StdRng};

#[derive(Clone, Copy)]
pub struct Bot {
    piece_values: [i32; 6],
    node_count: i64,
}

static RANDOMNESS: i64 = 10;

impl Bot {
    pub fn new() -> Bot {
        Bot {
            piece_values: [100, 300, 300, 500, 900, 100000],
            node_count: 0,
        }
    }
}

impl ChessBot for Bot {
    fn get_best_move(&mut self, board: &mut Board, rng: &mut StdRng) -> Move {
        let moves = board.get_legal_moves().clone().unwrap();

        let mut best_move: Move = moves[0];
        let mut best_value: i64 = -999999999;

        for m in moves {
            if !board.make_move(m, true) {
                panic!("Make move failed!\nMove: {}\nPosition: {}", m.to_string(), board.generate_fen());
            }

            let mut value = 0;

            match board.get_game_result_simplified() {
                0 => { }
                1 => {
                    board.undo_move(true);
                    if board.get_white_to_move() { return m; }
                    else { panic!("Invalid move played!\nFEN: {}\nPGN:\n{}", board.generate_fen(), board.get_pgn(None, None, None)); }
                }
                2 => {
                    board.undo_move(true);
                    if board.get_white_to_move() {
                        panic!("Invalid move played!\nFEN: {}\nPGN:\n{}", board.generate_fen(), board.get_pgn(None, None, None));
                    }
                    else { return m; }
                }
                -1 => {
                    value = -self.search(board, 3, rng);
                }
                _ => { }
            }

            board.undo_move(true).expect("Undo move failed");

            if value > best_value {
                best_value = value;
                best_move = m;
            }
        }

        println!("Node count: {}", self.node_count);
        self.node_count = 0;

        best_move
    }
}

impl Bot {
    fn search(&mut self, board: &mut Board, depth: i32, rng: &mut StdRng) -> i64 {
        if depth == 0 {
            return self.evaluate(board, rng);
        }

        let moves = board.get_legal_moves().clone().unwrap();
        let mut best_value = -99999999;

        for m in moves {
            board.make_move(m, true);

            let mut value: i64 = 0;

            match board.get_game_result_simplified() {
                0 => { }
                1 => {
                    board.undo_move(true);
                    if board.get_white_to_move() { return 99999999; }
                    else { panic!("Invalid move played!\nFEN: {}\nPGN:\n{}", board.generate_fen(), board.get_pgn(None, None, None)); }
                }
                2 => {
                    board.undo_move(true);
                    if board.get_white_to_move() {
                        panic!("Invalid move played!\nFEN: {}\nPGN:\n{}", board.generate_fen(), board.get_pgn(None, None, None));
                    }
                    else { return 99999999; }
                }
                -1 => {
                    value = -self.search(board, depth - 1, rng);
                }
                _ => { }
            }

            board.undo_move(true).expect("Failed to undo move");

            if value > best_value {
                best_value = value;
            }
        }

        best_value
    }

    //Returns the eval from the perspective of the side to move
    pub fn evaluate(&mut self, board: &mut Board, rng: &mut StdRng) -> i64 {
        let squares = board.get_squares();
        let mut white_value: i64 = 0;
        let mut black_value: i64 = 0;

        self.node_count += 1;

        for i in 0..64 {
            if squares[i].is_none() {
                continue;
            }

            let p = squares[i].unwrap();

            if p.white_piece {
                white_value += self.piece_values[p.piece_type as usize - 1] as i64;
            }
            else {
                black_value += self.piece_values[p.piece_type as usize - 1] as i64;
            }
        }

        let mut value = white_value - black_value;

        if !board.get_white_to_move() {
            value *= -1;
        }

        value + rng.random_range(0..RANDOMNESS)
    }
}