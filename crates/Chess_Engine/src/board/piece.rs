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
}