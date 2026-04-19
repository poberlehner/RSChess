#[derive(PartialEq, Copy, Clone)]
pub struct GameState
{
    pub white_to_move: bool,
    ///white then black, kingside then queenside
    pub castling_rights: [bool; 4],
    ///column, row
    pub en_passant_square: Option<(u8, u8)>,
    pub fifty_move_counter: u32,
    pub next_move_number: u32,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GameResult {
    NotStarted,
    InProgress,
    WhiteIsMated,
    BlackIsMated,
    Stalemate,
    Repetition,
    FiftyMoveRule,
    InsufficientMaterial,
    WhiteTimeout,
    BlackTimeout,
}