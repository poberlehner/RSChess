use crate::board::Board;
use crate::board::piece::{Piece, PieceType};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Move {
    pub origin_square: (u8, u8),
    pub destination_square: (u8, u8),

    pub promotion_type: PieceType,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct LogMove {
    pub origin_square: (u8, u8),
    pub destination_square: (u8, u8),
    pub promotion_type: PieceType,
    
    pub moved_piece: Piece,
    pub captured_piece: Option<Piece>,

    pub time_taken: std::time::Duration,

    pub en_passant: bool,
    pub en_passant_square_before: Option<(u8, u8)>,
    pub castling: bool,
    pub castling_rights_before: [bool; 4],
    pub fifty_move_counter_before: u32,
}

impl Move {
    pub fn to_string(&self) -> String {
        let mut string: String = String::from("");

        string.push_str(Board::square_coords_to_string(self.origin_square).as_str());
        string.push_str(Board::square_coords_to_string(self.destination_square).as_str());
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
}