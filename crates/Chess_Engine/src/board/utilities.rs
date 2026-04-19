use crate::board::gamestate::GameResult;
use crate::board::piece::{Piece, PieceType};
use crate::board::Board;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimeControl
{
    Unlimited,
    ///Duration for white then black
    Limited(Duration, Duration),
}

#[macro_export] macro_rules! square_contains {
    ($board:ident, $x:expr, $y:expr, $white:expr, $piece_type:expr) => {
        if ($board).squares[$x as usize][$y as usize].is_none() {
             false
        }
        else {
            if ($board).squares[$x as usize][$y as usize].unwrap().white_piece == $white as bool {
                if ($board).squares[$x as usize][$y as usize].unwrap().piece_type == $piece_type as PieceType {
                     true
                }
                else {
                    false
                }
            }
            else {
                false
            }
        }
    }
}

impl Board {
    #[inline(always)]
    pub fn column_char_to_index(column: char) -> Option<usize> {
        match column {
            'a' => { Some(0) }
            'b' => { Some(1) }
            'c' => { Some(2) }
            'd' => { Some(3) }
            'e' => { Some(4) }
            'f' => { Some(5) }
            'g' => { Some(6) }
            'h' => { Some(7) }
            _ => { None }
        }
    }

    //The return value is always lowercase
    #[inline(always)]
    pub fn column_index_to_char(column: i32) -> Option<char> {
        match column {
            0 => { Some('a') }
            1 => { Some('b') }
            2 => { Some('c') }
            3 => { Some('d') }
            4 => { Some('e') }
            5 => { Some('f') }
            6 => { Some('g') }
            7 => { Some('h') }
            _ => { None }
        }
    }

    #[inline(always)]
    pub fn square_coords_to_chars(coords: (u8, u8)) -> [char; 2] {
        //TODO: Analyze removing the quick fix for inverting the indices to avoid having to change the default positions of white and black
        let mut result: [char; 2] = ['0', '0'];

        result[0] = Self::column_index_to_char(coords.0 as i32).expect("Invalid coordinate");
        result[1] = char::from_digit(((7 - coords.1 as i32) + 1) as u32, 10).expect("Invalid coordinate");

        result
    }

    #[inline(always)]
    pub fn square_coords_to_string(coords: (u8, u8)) -> String {
        let mut result: String = String::from("");

        result.push(Self::column_index_to_char(coords.0 as i32).expect("Invalid coordinate"));
        result.push(char::from_digit(((7 - coords.1 as i32) + 1) as u32, 10).expect("Invalid coordinate"));

        result
    }

    #[inline(always)]
    pub fn index_from_coord(coords: (u8, u8)) -> i64 {
        (coords.1 * 8 + coords.0) as i64
    }

    #[inline(always)]
    pub fn piece_to_char(piece: Piece) -> Option<char> {
        match piece.piece_type {
            PieceType::King => { if piece.white_piece { Some('K') } else { Some('k') } },
            PieceType::Queen => { if piece.white_piece { Some('Q') } else { Some('q') } },
            PieceType::Rook => { if piece.white_piece { Some('R') } else { Some('r') } },
            PieceType::Knight => { if piece.white_piece { Some('N') } else { Some('n') } },
            PieceType::Bishop => { if piece.white_piece { Some('B') } else { Some('b') } },
            PieceType::Pawn => { if piece.white_piece { Some('P') } else { Some('p') } },
            _ => { None }
        }
    }

    #[inline(always)]
    pub fn char_to_piece(ch: char) -> Option<Piece> {
        let c = ch.to_lowercase().last().unwrap();

        match c {
            'k' => { Some(Piece { piece_type: PieceType::King, white_piece: ch.is_uppercase() }) }
            'q' => { Some(Piece { piece_type: PieceType::Queen, white_piece: ch.is_uppercase() }) }
            'r' => { Some(Piece { piece_type: PieceType::Rook, white_piece: ch.is_uppercase() }) }
            'n' => { Some(Piece { piece_type: PieceType::Knight, white_piece: ch.is_uppercase() }) }
            'b' => { Some(Piece { piece_type: PieceType::Bishop, white_piece: ch.is_uppercase() }) }
            'p' => { Some(Piece { piece_type: PieceType::Pawn, white_piece: ch.is_uppercase() }) }
            _ => { None }
        }
    }

    pub fn checkmate(&mut self) -> bool {
        let king_pos = if self.game_state.white_to_move { self.white_king_pos } else { self.black_king_pos };
        let attack_bitboard = self.get_attacked_squares_bitboard(!self.game_state.white_to_move);
        if ((1 << Self::index_from_coord(king_pos)) & attack_bitboard) != 0 {
            let moves = self.get_legal_moves().as_ref().expect("Failed to get legal moves");

            if moves.len() == 0 {
                return true;
            }
            else {
                return false;
            }
        }

        false
    }

    pub fn check(&mut self) -> bool {
        let king_pos = if self.game_state.white_to_move { self.white_king_pos } else { self.black_king_pos };

        let attack_bitboard = self.get_attacked_squares_bitboard(!self.game_state.white_to_move);

        if (attack_bitboard & (1 << Self::index_from_coord((king_pos.0, king_pos.1)))) != 0 {
            return true
        }

        false
    }

    pub fn load_fen(&mut self, fen: &str) {
        #[cfg(debug_assertions)]
        let start_time = std::time::Instant::now();
        self.squares = [[None; 8]; 8];

        self.game_state.castling_rights = [false; 4];
        self.game_state.en_passant_square = None;
        self.game_state.fifty_move_counter = 0;
        self.game_state.next_move_number = 1;

        let mut index: usize = 0;
        let chars = fen.chars().collect::<Vec<char>>();

        //---------------Group 1--------------
        let mut target_row: usize = 0;
        let mut target_col: usize = 0;

        for char in fen.chars() {
            index += 1;
            let white: bool = char.is_uppercase();

            if char.is_numeric() {
                target_col += char.to_digit(10).unwrap() as usize;
            }

            let lowercase = char.to_lowercase().next().unwrap();

            match lowercase {
                '/' => {
                    target_row += 1;
                    target_col = 0;
                }
                'k' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::King, white_piece: white });

                    if white {
                        self.white_king_pos = (target_col as u8, target_row as u8);
                    }
                    else {
                        self.black_king_pos = (target_col as u8, target_row as u8);
                    }

                    target_col += 1;
                }
                'n' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::Knight, white_piece: white });
                    target_col += 1;
                }
                'b' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::Bishop, white_piece: white });
                    target_col += 1;
                }
                'r' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::Rook, white_piece: white });
                    target_col += 1;
                }
                'q' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::Queen, white_piece: white });
                    target_col += 1;
                }
                'p' => {
                    self.squares[target_col][target_row] = Some(Piece { piece_type: PieceType::Pawn, white_piece: white });
                    target_col += 1;
                }
                ' ' => {
                    break;
                }
                _ => {}
            }
        }
        //---------------End Group 1-----------------------

        self.game_state.white_to_move = false;
        if chars[index] == 'w' {
            self.game_state.white_to_move = true;
        }
        //----------------End Group 2-------------------

        index += 2;

        let group2_start_index = index;
        for i in 0..4 {
            match chars[i + group2_start_index] {
                'K' => {
                    self.game_state.castling_rights[0] = true;
                }
                'Q' => {
                    self.game_state.castling_rights[1] = true;
                }
                'k' => {
                    self.game_state.castling_rights[2] = true;
                }
                'q' => {
                    self.game_state.castling_rights[3] = true;
                }
                '-' => {
                    index += 1;
                    break;
                }
                ' ' => {
                    break;
                }
                _ => {

                }
            }
            index += 1;
        }
        //-------------End Group 3-------------------

        index += 1;

        if chars[index] == '-' {
            index += 2;
        }
        else {
            self.game_state.en_passant_square = Some((Board::column_char_to_index(chars[index]).expect("Invalid column letter in FEN string") as u8, chars[index + 1].to_digit(10).expect("Not a valid row index in FEN String") as u8));
            index += 3;
        }

        //--------------End Group 4------------------

        if chars[index + 1] == ' ' {
            self.game_state.fifty_move_counter = chars[index].to_digit(10).expect("Invalid fifty move counter value in FEN string");
            index += 2;
        }
        else {
            self.game_state.fifty_move_counter = chars[index].to_digit(10).expect("Invalid fifty move counter value in FEN string") * 10 + chars[index + 1].to_digit(10).expect("Invalid fifty move counter value in FEN string");
            index += 3;
        }

        //---------------End Group 5----------------

        let next_move_count_string = &chars[index..chars.len()];
        self.game_state.next_move_number = next_move_count_string.iter().collect::<String>().parse::<u32>().expect("Invalid next move number in FEN string");

        self.invalidate_caches();

        self.game_result = GameResult::NotStarted;

        #[cfg(debug_assertions)]
        println!("Fen loading time: {}µs", start_time.elapsed().as_micros());
    }

    #[allow(unused)]
    pub fn generate_fen(&self) -> String {
        #[cfg(debug_assertions)]
        let start_time = std::time::Instant::now();
        let mut fen: String = String::from("");

        //Go back to front when switching to a flattened Array due to a wrong color positioning
        for y in 0..8 {
            let mut empty_squares = 0;
            for x in 0..8 {
                if self.squares[x][y].is_some() {
                    if empty_squares != 0 {
                        fen.push_str(&format!("{}", empty_squares));
                    }

                    empty_squares = 0;

                    fen.push(Self::piece_to_char(self.squares[x][y].unwrap()).expect(&format!("Invalid piece on square ({}, {})", x, y)));
                }
                else {
                    empty_squares += 1;
                }
            }

            if empty_squares != 0 {
                fen.push_str(&format!("{}", empty_squares));
            }

            if y != 7 {
                fen.push('/')
            }
        }

        fen.push(' ');

        fen.push( if self.game_state.white_to_move { 'w' } else { 'b' });

        fen.push(' ');

        {
            let mut any_castling_allowed: bool = false;
            if self.game_state.castling_rights[0] {
                fen.push('K');
                any_castling_allowed = true;
            }
            if self.game_state.castling_rights[1] {
                fen.push('Q');
                any_castling_allowed = true;
            }
            if self.game_state.castling_rights[2] {
                fen.push('k');
                any_castling_allowed = true;
            }
            if self.game_state.castling_rights[3] {
                fen.push('q');
                any_castling_allowed = true;
            }

            if !any_castling_allowed {
                fen.push('-');
            }
        }

        fen.push(' ');

        if self.game_state.en_passant_square.is_some() {
            fen.push(Self::column_index_to_char(self.game_state.en_passant_square.unwrap().0 as i32).expect("Invalid column in en passant square"));
            fen.push(self.game_state.en_passant_square.unwrap().1.to_string().pop().unwrap());
        }
        else {
            fen.push('-');
        }

        fen.push(' ');

        fen.push_str(self.game_state.fifty_move_counter.to_string().as_str());

        fen.push(' ');

        fen.push_str(self.game_state.next_move_number.to_string().as_str());

        #[cfg(debug_assertions)]
        println!("Fen generation time: {}µs", start_time.elapsed().as_micros());

        fen
    }
}