use std::panic;

use chess_engine::board::r#move::Move;
use chess_engine::bot::ChessBot;
use chess_engine::board::Board;
use rand::random;

pub struct Bot {
    piece_values: [i32; 6],
    node_count: i64,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            piece_values: [100, 300, 300, 500, 900, 100000],
            node_count: 0
        }
    }
}

impl ChessBot for Bot {
    fn get_best_move(&mut self, board: &mut Board) -> Move {
        let moves = board.get_legal_moves().clone().unwrap();

        let mut best_move: Move = moves[0];
        let mut best_value: i64 = -999999999;
        let mut beta: i64 = 999999999;

        for m in moves {
            if !board.make_move(m, true) {
                panic!("Make move failed!\nMove: {}\nPosition: {}", m.to_string(), board.generate_fen());
            }

            let value = -self.search(board, 3);

            board.undo_move(true);

            if value >= beta {
                beta = value;
            }
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
    fn search(&mut self, board: &mut Board, depth: i32) -> i64 {
        if depth == 0 {
            return self.evaluate(board);
        }

        let moves = board.get_legal_moves().clone().unwrap();
        let mut best_value = -99999999;

        for m in moves {
            if !board.make_move(m, true) {
                match board.get_game_result_simplified() {
                    0 => { return 0; }
                    1 => {
                        if board.get_white_to_move() { return 99999999; }
                        else { return -99999999; }
                    }
                    2 => {
                        if board.get_white_to_move() { return -99999999; }
                        else { return 99999999; }
                    }
                    -1 => {
                        panic!("Make move failed!\nMove: {}\nPosition: {}", m.to_string(), board.generate_fen());
                    }
                    _ => { }
                }
            }

            let value: i64;

            value = -self.search(board, depth - 1);

            board.undo_move(true);

            if value > best_value {
                best_value = value;
            }
        }

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

        value + random::<i64>() % 10
    }

    #[allow(unused)]
    fn get_piece_num(&mut self, board: &Board) -> i32 {
        let squares = board.get_squares();
        let mut result = 0;

        for i in 0..64 {
            if squares[i].is_some() {
                result += 1;
            }
        }

        result
    }
}