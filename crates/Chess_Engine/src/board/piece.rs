#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    None = 0,
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub white_piece: bool
}

impl Piece {
    pub fn as_num(&self) -> usize {
        if self.piece_type == PieceType::None {
            return 0;
        }

        self.piece_type as usize + 7 * self.white_piece as usize
    }

    pub fn to_char(&self) -> char {
        match self.piece_type {
            PieceType::King => { if self.white_piece { 'K' } else { 'k' } }
            PieceType::Queen => { if self.white_piece { 'Q' } else { 'q' } }
            PieceType::Rook => { if self.white_piece { 'R' } else { 'r' } }
            PieceType::Knight => { if self.white_piece { 'N' } else { 'n' } }
            PieceType::Bishop => { if self.white_piece { 'B' } else { 'b' } }
            PieceType::Pawn => { if self.white_piece { 'P' } else { 'p' } }
            PieceType::None => { '0' }
        }
    }

    pub fn to_char_uppercase(&self) -> char {
        match self.piece_type {
            PieceType::King => { 'K' }
            PieceType::Queen => { 'Q' }
            PieceType::Rook => { 'R' }
            PieceType::Knight => { 'N' }
            PieceType::Bishop => { 'B' }
            PieceType::Pawn => { 'P' }
            PieceType::None => { '0' }
        }
    }
}