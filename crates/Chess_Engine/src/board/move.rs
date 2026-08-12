use std::time::Duration;
use crate::board::Board;
use crate::board::piece::{Piece, PieceType};

pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
    Northeast = 4,
    Southeast = 5,
    Southwest = 6,
    Northwest = 7
}

pub static DIR_INDEX_TO_OFFSET: [i32; 8] = [8, 1, -8, -1, 9, -7, -9, 7];
pub static KNIGHT_JUMP_OFFSETS: [i32; 8] = [17, 15, 10, 6, -6, -10, -15, -17];

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Move {
    pub origin_square: u8,
    pub destination_square: u8,

    pub promotion_type: PieceType,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct LogMove {
    pub origin_square: u8,
    pub destination_square: u8,
    pub promotion_type: PieceType,

    pub moved_piece: Piece,
    pub captured_piece: Option<Piece>,

    pub time_taken: std::time::Duration,

    pub en_passant: bool,
    pub en_passant_square_before: Option<u8>,
    pub castling: bool,
    pub castling_rights_before: [bool; 4],
    pub fifty_move_counter_before: u32,

    pub null_move: bool
}

impl Move {
    ///Returns the move name in UCI format
    /// Example: "e7e8q"
    pub fn to_string(&self) -> String {
        let mut string: String = String::from("");

        string.push_str(Board::square_index_to_string(self.origin_square).as_str());
        string.push_str(Board::square_index_to_string(self.destination_square).as_str());
        if self.promotion_type != PieceType::None && self.promotion_type != PieceType::King {
            string.push( match self.promotion_type {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => ' '
            })
        }

        string
    }

    ///string must be in the UCI format
    pub fn from_str(string: &str) -> Move {
        let mut m = Move { origin_square: 0, destination_square: 0, promotion_type: PieceType::None };

        let chars = string.chars().collect::<Vec<char>>();
        m.origin_square = Board::file_char_to_index(chars[0]).unwrap() as u8 + (chars[1].to_digit(10).unwrap() - 1) as u8 * 8;
        m.destination_square = Board::file_char_to_index(chars[2]).unwrap() as u8 + (chars[3].to_digit(10).unwrap() - 1) as u8 * 8;
        if chars.len() == 5 {
            m.promotion_type = Board::char_to_piece(chars[4]).unwrap().piece_type;
        }

        m
    }
}

impl LogMove {
    pub fn to_move(&self) -> Move {
        return Move {
            origin_square: self.origin_square,
            destination_square: self.destination_square,
            promotion_type: self.promotion_type
        }
    }

    pub fn null_move() -> Self {
        LogMove {
            null_move: true,
            origin_square: 0,
            destination_square: 0,
            promotion_type: PieceType::None,
            captured_piece: None,
            castling: false,
            castling_rights_before: [false, false, false, false],
            en_passant: false,
            en_passant_square_before: None,
            fifty_move_counter_before: 1,
            moved_piece: Piece { piece_type: PieceType::None, white_piece: true },
            time_taken: Duration::from_hours(0)
        }
    }
}