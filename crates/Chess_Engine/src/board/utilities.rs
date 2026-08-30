use log::*;
use crate::board::gamestate::GameResult;
use crate::board::piece::{Piece, PieceType};
use crate::board::Board;
use std::time::Duration;
use std::fmt::Write as _;
#[cfg(debug_assertions)]
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimeControl
{
    Unlimited,
    ///Time then increment
    Limited(Duration, Duration),
}

#[macro_export] macro_rules! square_contains {
    ($board:ident, $i:expr, $white:expr, $piece_type:expr) => {
        if ($board).squares[$i as usize].is_none() {
             false
        }
        else {
            if ($board).squares[$i as usize].unwrap().white_piece == $white as bool {
                if ($board).squares[$i as usize].unwrap().piece_type == $piece_type as PieceType {
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
    pub fn column_char_to_index(column: char) -> usize {
        match column {
            'a' => { 0 }
            'b' => { 1 }
            'c' => { 2 }
            'd' => { 3 }
            'e' => { 4 }
            'f' => { 5 }
            'g' => { 6 }
            'h' => { 7 }
            _ => {
                error!("Tried to convert invalid column char '{}' to an index", column);
                0
            }
        }
    }

    #[inline(always)]
    pub fn file_char_to_index(file: char) -> usize {
        Self::column_char_to_index(file)
    }

    #[inline(always)]
    pub fn valid_file_char(c: char) -> bool {
        c.is_ascii_digit()
    }

    #[inline(always)]
    pub fn valid_column_char(c: char) -> bool {
        Self::valid_file_char(c)
    }

    //The return value is lowercase
    #[inline(always)]
    pub fn column_index_to_char(column: i32) -> char {
        match column {
            0 => { 'a' }
            1 => { 'b' }
            2 => { 'c' }
            3 => { 'd' }
            4 => { 'e' }
            5 => { 'f' }
            6 => { 'g' }
            7 => { 'h' }
            _ => {
                error!("Tried to convert invalid column index '{}'to char", column);
                'a'
            }
        }
    }

    #[inline(always)]
    pub fn file_index_to_char(file: i32) -> char {
        Self::column_index_to_char(file)
    }

    #[inline(always)]
    pub fn square_index_to_chars(index: u8) -> [char; 2] {
        let mut result: [char; 2] = ['0', '0'];

        result[0] = Self::column_index_to_char((index % 8) as i32);
        let rank = char::from_digit(((index / 8) as i32 + 1) as u32, 10);
        if rank.is_none() {
            warn!("Unable to convert index {index} to chars");
        }

        result[1] = rank.unwrap_or('0');

        result
    }

    #[inline(always)]
    pub fn square_index_to_string(index: u8) -> String {
        let mut result: String = String::from("");

        result.push(Self::column_index_to_char((index % 8) as i32));
        let rank = char::from_digit(((index / 8) as i32 + 1) as u32, 10);
        if rank.is_none() {
            warn!("Unable to convert index {index} to a string");
        }

        result.push(rank.unwrap_or('0'));

        result
    }

    #[inline(always)]
    pub fn index_from_coord(coords: (u8, u8)) -> i64 {
        (coords.1 * 8 + coords.0) as i64
    }

    #[inline(always)]
    pub fn piece_to_char(piece: Piece) -> char {
        match piece.piece_type {
            PieceType::King => { if piece.white_piece { 'K' } else { 'k' } },
            PieceType::Queen => { if piece.white_piece { 'Q' } else { 'q' } },
            PieceType::Rook => { if piece.white_piece { 'R' } else { 'r' } },
            PieceType::Knight => { if piece.white_piece { 'N' } else { 'n' } },
            PieceType::Bishop => { if piece.white_piece { 'B' } else { 'b' } },
            PieceType::Pawn => { if piece.white_piece { 'P' } else { 'p' } },
            PieceType::None => {
                warn!("Converting PieceType::None to a char ('0') which might lead to unintended behaviour");
                '0'
            }
        }
    }

    #[inline(always)]
    pub fn char_to_piece(ch: char) -> Piece {
        let c = ch.to_lowercase().last();

        if c.is_none() {
            warn!("Unable to make char '{ch}' lowercase");
        }

        let c = c.unwrap_or('p');

        match c {
            'k' => { Piece { piece_type: PieceType::King, white_piece: ch.is_uppercase() } }
            'q' => { Piece { piece_type: PieceType::Queen, white_piece: ch.is_uppercase() } }
            'r' => { Piece { piece_type: PieceType::Rook, white_piece: ch.is_uppercase() } }
            'n' => { Piece { piece_type: PieceType::Knight, white_piece: ch.is_uppercase() } }
            'b' => { Piece { piece_type: PieceType::Bishop, white_piece: ch.is_uppercase() } }
            'p' => { Piece { piece_type: PieceType::Pawn, white_piece: ch.is_uppercase() } }
            _ => {
                error!("Unable to convert invalid char '{}' to a chess piece", ch);
                Piece { piece_type: PieceType::None, white_piece: false }
            }
        }
    }

    pub fn checkmate(&mut self) -> bool {
        let king_pos = if self.game_state.white_to_move { self.white_king_pos } else { self.black_king_pos };
        let attack_bitboard = self.get_attacked_squares_bitboard(!self.game_state.white_to_move);
        if ((1 << king_pos) & attack_bitboard) != 0 {
            #[allow(clippy::unwrap_used, reason = "get_legal_moves always returns a reference to a Some value")]
            let moves = self.get_legal_moves().as_ref().unwrap();

            return moves.is_empty();
        }

        false
    }

    pub fn check(&mut self) -> bool {
        let king_pos = if self.game_state.white_to_move { self.white_king_pos } else { self.black_king_pos };

        let attack_bitboard = self.get_attacked_squares_bitboard(!self.game_state.white_to_move);

        (attack_bitboard & (1 << king_pos)) != 0
    }

    pub fn load_fen(&mut self, fen: &str) {
        #[cfg(debug_assertions)]
        let start_time = Instant::now();

        self.start_fen = fen.to_string();
        self.squares = [None; 64];

        self.game_state.castling_rights = [false; 4];
        self.game_state.en_passant_square = None;
        self.game_state.fifty_move_counter = 0;
        self.game_state.next_move_number = 1;

        let mut index: usize = 0;
        let chars = fen.chars().collect::<Vec<char>>();

        //---------------Group 1--------------
        let mut target_row: usize = 7;
        let mut target_col: usize = 0;

        for char in fen.chars() {
            index += 1;
            let white: bool = char.is_uppercase();

            if char.is_numeric() && char.is_ascii() {
                target_col += char.to_digit(10).unwrap_or(0) as usize;
            }

            let lowercase = char.to_lowercase().next();
            if lowercase.is_none() {
                warn!("Failed to change char {char} to lowercase as index {index} in fen string \"{fen}\"");
            }
            let lowercase = lowercase.unwrap_or('/');

            match lowercase {
                '/' => {
                    target_row -= 1;
                    target_col = 0;
                }
                'k' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::King, white_piece: white });

                    if white {
                        self.white_king_pos = (target_col + target_row * 8) as u8;
                    }
                    else {
                        self.black_king_pos = (target_col + target_row * 8) as u8;
                    }

                    target_col += 1;
                }
                'n' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::Knight, white_piece: white });
                    target_col += 1;
                }
                'b' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::Bishop, white_piece: white });
                    target_col += 1;
                }
                'r' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::Rook, white_piece: white });
                    target_col += 1;
                }
                'q' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::Queen, white_piece: white });
                    target_col += 1;
                }
                'p' => {
                    self.squares[target_col + target_row * 8] = Some(Piece { piece_type: PieceType::Pawn, white_piece: white });
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
            if Self::valid_file_char(chars[index]) {
                warn!("Invalid file char on index {index} of fen string \"{fen}\"");
            }

            let rank_char = chars[index + 1].to_digit(10);

            if rank_char.is_none() {
                warn!("Invalid rank char '{}' at index {} of fen string \"{fen}\"", chars[index + 1], index + 1);
            }

            let rank_char = rank_char.unwrap_or(0) as u8;

            self.game_state.en_passant_square = Some(Board::column_char_to_index(chars[index]) as u8 +
                8 * rank_char);
            index += 3;
        }

        //--------------End Group 4------------------

        let fifty_move_counter = chars[index].to_digit(10);
        if fifty_move_counter.is_none() {
            warn!("Invalid first char of the fifty-move-counter: '{}'", chars[index]);
        }

        let fifty_move_counter = fifty_move_counter.unwrap_or(0);
        if chars[index + 1] == ' ' {
            self.game_state.fifty_move_counter = fifty_move_counter;
            index += 2;
        }
        else {
            let fifty_move_counter_second = chars[index + 1].to_digit(10);

            if fifty_move_counter_second.is_none() {
                warn!("Invalid second fifty move counter char: '{}'", chars[index + 1]);
            }

            let fifty_move_counter_second = fifty_move_counter_second.unwrap_or(0);
            self.game_state.fifty_move_counter = fifty_move_counter * 10 + fifty_move_counter_second;
            index += 3;
        }

        //---------------End Group 5----------------

        let next_move_count_string = &chars[index..chars.len()];
        let next_move_count = &chars[index..chars.len()].iter().collect::<String>().parse::<u32>();

        if let Err(err) = next_move_count {
            warn!("Invalid next move count \"{}\" in fen string \"{fen}\" with error \"{err}\"", next_move_count_string.iter().collect::<String>());
        }

        self.game_state.next_move_number = next_move_count.clone().unwrap_or(1);

        self.invalidate_caches();

        #[cfg(debug_assertions)]
        trace!("Load fen string \"{}\" in {}µs", fen, start_time.elapsed().as_micros());

        self.game_result = GameResult::NotStarted;
    }

    #[allow(unused)]
    pub fn generate_fen(&self) -> String {
        #[cfg(debug_assertions)]
        let start_time = Instant::now();
        let mut fen: String = String::from("");

        for y in 0..8 {
            let mut empty_squares = 0;
            for x in 0..8 {
                if let Some(square) = self.squares[x + (7 - y) * 8] {
                    if empty_squares != 0 {
                        let _ = write!(fen, "{}", empty_squares);
                    }

                    empty_squares = 0;

                    fen.push(Self::piece_to_char(square));
                }
                else {
                    empty_squares += 1;
                }
            }

            if empty_squares != 0 {
                let _ = write!(fen, "{}", empty_squares);
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

        if let Some(en_passant_square) = self.game_state.en_passant_square {
            if en_passant_square % 8 >= 8 {
                warn!("Invalid en passant square");
            }
            else {
                fen.push(Self::column_index_to_char((en_passant_square % 8) as i32));
                #[allow(clippy::unwrap_used)]
                fen.push((en_passant_square / 8).to_string().pop().unwrap());
            }
        }
        else {
            fen.push('-');
        }

        fen.push(' ');

        fen.push_str(self.game_state.fifty_move_counter.to_string().as_str());

        fen.push(' ');

        fen.push_str(self.game_state.next_move_number.to_string().as_str());

        #[cfg(debug_assertions)]
        trace!("Generated fen string \"{}\" in {}µs", fen, start_time.elapsed().as_micros());

        fen
    }
}