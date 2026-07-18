use raylib::ffi::TraceLogLevel::LOG_INFO;
use raylib::trace_log;

use crate::board::gamestate::GameResult;
use crate::board::piece::{Piece, PieceType};
use crate::board::r#move::{LogMove, Move};
use crate::board::Board;
use crate::board::utilities::TimeControl;
use std::cmp::max;
use std::time::*;

//Main api functions
impl Board {
    ///returns: success
    pub fn make_move(&mut self, m: Move, in_search: bool) -> bool {
        if self.game_result != GameResult::InProgress {
            if !in_search {
                println!("The game is not running");
                println!("Game Result: {:?}", self.game_result);
            }

            return false;
        }

        if self.legal_moves.is_none() {
            self.get_legal_moves();
        }

        if !self.legal_moves.as_ref().unwrap().contains(&m) {
            println!("Position: {}", self.generate_fen());
            println!("The Move: {} is not legal", m.to_string());
            return false;
        }

        if !self.handle_time() {
            println!("No time left");
            return false;
        }

        let mut current_logmove: LogMove = LogMove {
            origin_square: m.origin_square,
            destination_square: m.destination_square,
            promotion_type: m.promotion_type,
            time_taken: self.move_start_time.elapsed(),
            castling: false,
            captured_piece: None,
            en_passant: false,
            en_passant_square_before: self.game_state.en_passant_square,
            castling_rights_before: self.game_state.castling_rights,
            fifty_move_counter_before: self.game_state.fifty_move_counter,
            moved_piece: self.squares[m.origin_square as usize].unwrap()
        };

        if self.time_control != TimeControl::Unlimited {
            self.move_start_time = Instant::now();
        }

        let mut special_move = false;

        //Handle the fifty move rule
        {
            self.game_state.fifty_move_counter += 1;

            if !self.game_state.white_to_move {
                self.game_state.next_move_number += 1;
            }

            //Reset on pawn move
            if self.squares[m.origin_square as usize]
                .expect("No piece at the Origin of the Move").piece_type == PieceType::Pawn {
                self.game_state.fifty_move_counter = 0;
            }

            //Reset on capture
            if self.squares[m.destination_square as usize].is_some() {
                self.game_state.fifty_move_counter = 0;
            }

            if self.game_state.fifty_move_counter >= 100 {
                self.game_result = GameResult::FiftyMoveRule;
            }
        }

        special_move |= self.handle_en_passant(m, &mut current_logmove);

        //Promotion
        if m.promotion_type != PieceType::None {
            #[cfg(debug_assertions)]
            {
                if m.promotion_type == PieceType::King {
                    panic!("Promotion type is King");
                }
                if self.squares[m.origin_square as usize]
                    .expect("No piece at the Origin of the Move").piece_type != PieceType::Pawn {
                    panic!("Non pawn piece with a promotion type other than none");
                }
            }

            current_logmove.captured_piece = self.squares [m.destination_square as usize];
            self.squares [m.destination_square as usize] =
                Some(Piece { piece_type: m.promotion_type,
                    white_piece: self.squares [m.origin_square as usize].unwrap().white_piece});
            self.squares [m.origin_square as usize] = None;
        }
        else //Special move handling
        {
            if special_move {
                self.game_state.fifty_move_counter = 0;
            }

            special_move |= self.handle_castling(m, &mut current_logmove);

            if !special_move {
                current_logmove.captured_piece = self.squares [m.destination_square as usize];

                self.squares [m.destination_square as usize] =
                    self.squares [m.origin_square as usize];
                self.squares [m.origin_square as usize] = None;
            }

            if self.squares [m.destination_square as usize].unwrap().piece_type == PieceType::King {
                if self.game_state.white_to_move {
                    self.white_king_pos = m.destination_square;
                }
                else {
                    self.black_king_pos = m.destination_square;
                }
            }
        }

        self.game_state.white_to_move = !self.game_state.white_to_move;

        self.invalidate_caches();

        self.generate_all_bitboards();

        self.move_log.push(current_logmove);

       self.handle_end_conditions(in_search, &current_logmove);

        true
    }

    pub fn undo_move(&mut self, _in_search: bool) -> Option<LogMove> {
        if self.move_log.is_empty() {
            return None;
        }

        if self.game_state.white_to_move {
            self.game_state.next_move_number = max(1, self.game_state.next_move_number as i64 - 1) as u32;
        }

        self.game_state.white_to_move = !self.game_state.white_to_move;

        let last = self.move_log.pop().unwrap();
        let piece = self.squares[last.destination_square as usize].expect("Invalid logged move");

        if self.game_result != GameResult::BlackIsMated && self.game_result != GameResult::WhiteIsMated {
            self.zobrist_history.pop();
        }

        if piece.piece_type == PieceType::King {
            if piece.white_piece {
                self.white_king_pos = last.origin_square;
            }
            else {
                self.black_king_pos = last.origin_square;
            }
        }

        self.game_state.fifty_move_counter = last.fifty_move_counter_before;

        if last.promotion_type == PieceType::None {
            self.squares[last.origin_square as usize] =
                self.squares[last.destination_square as usize];
        }
        else {
            self.squares[last.origin_square as usize] =
                Some(Piece { white_piece: piece.white_piece, piece_type: PieceType::Pawn });
        }

        if !last.en_passant {
            self.squares[last.destination_square as usize] = last.captured_piece;
        }
        else {
            self.squares[last.destination_square as usize] = None;

            self.squares[(last.destination_square as i32 + if piece.white_piece { -8 } else { 8 }) as usize] =
                Some(Piece { piece_type: PieceType::Pawn, white_piece: !piece.white_piece });
        }

        if last.castling {
            //Black
            if last.origin_square == 60 {
                if last.destination_square == 62 {
                    self.squares[61] = None;
                    self.squares[63] = Some(Piece { piece_type: PieceType::Rook, white_piece: false});
                }
                else {
                    self.squares[59] = None;
                    self.squares[56] = Some(Piece { piece_type: PieceType::Rook, white_piece: false});
                }
            }
            //White
            else {
                if last.destination_square == 6 {
                    self.squares[5] = None;
                    self.squares[7] = Some(Piece { piece_type: PieceType::Rook, white_piece: true});
                }
                else {
                    self.squares[3] = None;
                    self.squares[0] = Some(Piece { piece_type: PieceType::Rook, white_piece: true});
                }
            }
        }

        self.game_state.en_passant_square = last.en_passant_square_before;

        self.game_state.castling_rights = last.castling_rights_before;

        if self.game_result != GameResult::InProgress {
            self.game_result = GameResult::InProgress;
        }

        self.invalidate_caches();

        self.generate_all_bitboards();

        Some(last)
    }

    pub fn start_game(&mut self, time_control: TimeControl) {
        if self.game_result != GameResult::InProgress {
            self.invalidate_caches();
            self.zobrist_history.clear();
            self.check = false;
            self.move_log.clear();
            self.move_start_time = Instant::now();

            self.game_result = GameResult::InProgress;

            self.time_control = time_control;
            self.white_used_time = Duration::new(0, 0);
            self.black_used_time = Duration::new(0, 0);

        }
    }

    pub fn get_game_result(&self) -> GameResult { self.game_result }

    pub fn get_time_control(&self) -> TimeControl { self.time_control }

    pub fn get_white_to_move(&self) -> bool { self.game_state.white_to_move }

    ///White then black, kingside then queenside
    pub fn get_castling_rights(&self) -> [bool; 4] { self.game_state.castling_rights }

    pub fn get_squares(&self) -> [Option<Piece>; 64] { self.squares }

    pub fn get_piece(&self, index: usize) -> Option<Piece> { self.squares[index] }

    ///Returns 0 when time_control is unlimited
    pub fn get_time_left(&self, white: bool) -> Duration {
        match self.time_control {
            TimeControl::Limited(white_time, black_time) => {
                if white {
                    if self.game_state.white_to_move {
                        if self.move_start_time.elapsed() > white_time - self.white_used_time {
                            return Duration::new(0, 0);
                        }

                        white_time - self.white_used_time - self.move_start_time.elapsed()
                    }
                    else {
                        white_time - self.white_used_time
                    }
                }
                else {
                    if self.game_state.white_to_move {
                        black_time - self.black_used_time
                    }
                    else {
                        if self.move_start_time.elapsed() > black_time - self.black_used_time {
                            return Duration::new(0, 0);
                        }

                        black_time - self.black_used_time - self.move_start_time.elapsed()
                    }
                }
            }
            TimeControl::Unlimited => {
                return Duration::new(0, 0);
            }
        }
    }

    ///0 == Draw; 1 = White won; 2 = Black won; -1 = Game not finished
    pub fn get_game_result_simplified(&self) -> i32 {
        return match self.game_result {
            GameResult::BlackIsMated  => { 1 }
            GameResult::BlackTimeout  => { 1 }
            GameResult::WhiteIsMated  => { 2 }
            GameResult::WhiteTimeout  => { 2 }
            GameResult::InProgress    => {-1 }
            GameResult::NotStarted    => {-1 }
            GameResult::InsufficientMaterial => { 0 }
            GameResult::FiftyMoveRule => { 0 }
            _ => {
                0
            }
        }
    }

    pub fn get_move_history(&self) -> String {
        //TODO
        return " ".to_string();
    }

    pub fn invalidate_caches(&mut self) {
        self.legal_moves = None;
        self.white_double_check = None;
        self.black_double_check = None;
        self.white_pieces_bitboard = None;
        self.black_pieces_bitboard = None;
        self.white_check_block_bitboard = None;
        self.black_check_block_bitboard = None;
        self.white_diagonal_pin_bitboard = None;
        self.black_diagonal_pin_bitboard = None;
        self.white_orthogonal_pin_bitboard = None;
        self.black_orthogonal_pin_bitboard = None;
        self.white_en_passant_pin_bitboard = None;
        self.black_en_passant_pin_bitboard = None;
        self.white_attacked_squares_bitboard = None;
        self.black_attacked_squares_bitboard = None;
    }
}

//Main API helper functions
impl Board {
    fn handle_castling(&mut self, m: Move, log_move: &mut LogMove) -> bool {
        if self.squares[m.origin_square as usize].is_some() &&
            self.squares[m.origin_square as usize].unwrap().piece_type == PieceType::King {
            if self.game_state.castling_rights[0] {
                //White kingside castle
                if  m.origin_square == 4 &&
                    m.destination_square == 6 &&
                    self.squares[5].is_none() && self.squares[6].is_none() {

                    self.squares[4] = None;
                    self.squares[7] = None;
                    self.squares[6] = Some(Piece { piece_type: PieceType::King, white_piece: true });
                    self.squares[5] = Some(Piece { piece_type: PieceType::Rook, white_piece: true });

                    self.game_state.castling_rights[0] = false;
                    self.game_state.castling_rights[1] = false;

                    log_move.castling = true;

                    return true;
                }
            }
            if self.game_state.castling_rights[1] {
                //White queenside castle
                if  m.origin_square == 4 &&
                    m.destination_square == 2 &&
                    self.squares[1].is_none() && self.squares[2].is_none() && self.squares[3].is_none() {

                    self.squares[4] = None;
                    self.squares[0] = None;
                    self.squares[2] = Some(Piece { piece_type: PieceType::King, white_piece: true });
                    self.squares[3] = Some(Piece { piece_type: PieceType::Rook, white_piece: true });

                    self.game_state.castling_rights[0] = false;
                    self.game_state.castling_rights[1] = false;

                    log_move.castling = true;

                    return true;
                }
            }
            if self.game_state.castling_rights[2] {
                //Black kingside castle
                if  m.origin_square == 60 &&
                    m.destination_square == 62 &&
                    self.squares[61].is_none() && self.squares[62].is_none() {

                    self.squares[60] = None;
                    self.squares[63] = None;
                    self.squares[62] = Some(Piece { piece_type: PieceType::King, white_piece: false });
                    self.squares[61] = Some(Piece { piece_type: PieceType::Rook, white_piece: false });

                    self.game_state.castling_rights[2] = false;
                    self.game_state.castling_rights[3] = false;

                    log_move.castling = true;

                    return true;
                }
            }
            if self.game_state.castling_rights[3] {
                //Black queenside castle
                if  m.origin_square == 60 &&
                    m.destination_square == 58 &&
                    self.squares[57].is_none() && self.squares[58].is_none() && self.squares[59].is_none() {

                    self.squares[60] = None;
                    self.squares[56] = None;
                    self.squares[58] = Some(Piece { piece_type: PieceType::King, white_piece: false });
                    self.squares[59] = Some(Piece { piece_type: PieceType::Rook, white_piece: false });

                    log_move.castling = true;

                    self.game_state.castling_rights[2] = false;
                    self.game_state.castling_rights[3] = false;

                    return true;
                }
            }
        }

        //Disable castling when a Rook is moved or captured
        if (m.destination_square == 7) || (m.origin_square == 7) {
            self.game_state.castling_rights[0] = false;
        }
        if (m.destination_square == 0) || (m.origin_square == 0) {
            self.game_state.castling_rights[1] = false;
        }
        if (m.destination_square == 63) || (m.origin_square == 63) {
            self.game_state.castling_rights[2] = false;
        }
        if (m.destination_square == 56) || (m.origin_square == 56) {
            self.game_state.castling_rights[3] = false;
        }
        if m.origin_square == 4 {
            self.game_state.castling_rights[0] = false;
            self.game_state.castling_rights[1] = false;
        }
        if m.origin_square == 60 {
            self.game_state.castling_rights[2] = false;
            self.game_state.castling_rights[3] = false;
        }

        false
    }

    fn handle_en_passant(&mut self, m: Move, log_move: &mut LogMove) -> bool {
        if self.squares[m.origin_square as usize].is_some() &&
            self.squares[m.origin_square as usize].unwrap().piece_type == PieceType::Pawn {

            if m.origin_square.abs_diff(m.destination_square) == 16 {
                self.game_state.en_passant_square = Some(if log_move.moved_piece.white_piece { m.origin_square + 8 } else { m.origin_square - 8 });

                return false;
            }

            //If en passant is played
            if self.game_state.en_passant_square.is_some() && m.destination_square == self.game_state.en_passant_square.unwrap() {
                self.squares[(self.game_state.en_passant_square.unwrap() as i32 + if log_move.moved_piece.white_piece { -8i32 } else { 8 }) as usize] = None;

                self.squares[m.destination_square as usize] =
                    Some(Piece { piece_type: PieceType::Pawn, white_piece: self.game_state.white_to_move });
                self.squares[m.origin_square as usize] = None;
                self.game_state.en_passant_square = None;

                log_move.en_passant = true;

                return true;
            }
        }


        self.game_state.en_passant_square = None;

        false
    }
}

///Handle End conditions
impl Board {
    #[allow(unused)]
    fn handle_end_conditions(&mut self, in_search: bool,  logmove: &LogMove) {
        // let red = "\x1b[31m";      // Red text
        // let green = "\x1b[32m";    // Green text
        // let yellow = "\x1b[33m";   // Yellow text
        let blue = "\x1b[34m";     // Blue text
        // let magenta = "\x1b[35m";  // Magenta text
        // let cyan = "\x1b[36m";     // Cyan text
        // let white = "\x1b[37m";    // White text
        let reset = "\x1b[0m";     // Reset to default

        if self.checkmate() {
            if self.game_state.white_to_move {
                self.game_result = GameResult::WhiteIsMated;
                if !in_search {
                    println!("{blue}White Is Mated{reset}");
                    trace_log(LOG_INFO, "White Is Mated");
                }

                return;
            }
            else {
                self.game_result = GameResult::BlackIsMated;
                if !in_search {
                    println!("{blue}Black Is Mated{reset}");
                    trace_log(LOG_INFO, "Black Is Mated");
                }

                return;
            }
        }

        //Handle repetition
        {
            self.zobrist_history.push(self.zobrist.generate_key(&self));

            let mut equals: i32 = 0;
            for i in 0..self.zobrist_history.len() - 1 {
                if self.zobrist_history[i] == self.zobrist_history[self.zobrist_history.len() - 1] {
                    equals += 1;
                }
            }

            if equals >= 2 {
                self.game_result = GameResult::Repetition;

                if !in_search {
                    println!("{blue}Draw due to repetition{reset}");
                    trace_log(LOG_INFO, "Draw due to repetition");
                }
            }
        }

        if !in_search {
            if self.game_state.fifty_move_counter == 50 {
                self.game_result = GameResult::FiftyMoveRule;
                println!("{blue}Draw by the fifty-move rule{reset}");
                trace_log(LOG_INFO, "Draw by the fifty-move rule");
            }

            //Handle insufficient material
            if self.has_insufficient_material(true) && self.has_insufficient_material(false) {
                self.game_result = GameResult::InsufficientMaterial;
                println!("{blue}Draw due to insufficient material{reset}");
                trace_log(LOG_INFO, "Draw due to insufficient material");
            }

            //Handle Stalemate
            if self.get_legal_moves().as_ref().unwrap().is_empty() {
                self.game_result = GameResult::Stalemate;
                println!("{blue}Draw due to stalemate{reset}");
                trace_log(LOG_INFO, "Draw due to stalemate");
            }
        }
    }

    fn has_insufficient_material(&self, white: bool) -> bool {
        //if one piece has been found that cannot checkmate with the king alone
        let mut insufficient_piece: bool = false;
        let mut knight_num: i32 = 0;
        let mut opp_pieces: i32 = 0;

        for i in 0..64 {
            if self.squares[i].is_some() {
                if self.squares[i].unwrap().white_piece == white {
                    if insufficient_piece == true {
                        return false;
                    }

                    match self.squares[i].unwrap().piece_type {
                        PieceType::Bishop => { insufficient_piece = true; }
                        PieceType::Knight => {
                            knight_num += 1;

                            if knight_num == 2 {
                                insufficient_piece = true;
                            }
                        }
                        PieceType::King => { }
                        _ => { return false; }
                    }
                }
                else {
                    opp_pieces += 1;
                }
            }
        }

        if knight_num == 2 && opp_pieces >= 2 {
            return false;
        }

        true
    }

    //Returns true if there is time left
    fn handle_time(&mut self) -> bool {
        match self.time_control {
            TimeControl::Limited(white, black) => {
                let blue = "\x1b[34m";     // Blue text
                let reset = "\x1b[0m";     // Reset to default

                if self.game_state.white_to_move {
                    self.white_used_time += self.move_start_time.elapsed();
                }
                else {
                    self.black_used_time += self.move_start_time.elapsed();
                }

                if self.white_used_time > white {
                    self.white_used_time = white;
                }
                if self.black_used_time > black {
                    self.black_used_time = black;
                }

                if self.white_used_time >= white {
                    self.game_result = GameResult::WhiteTimeout;
                    println!("{blue}Black won by white Timeout{reset}");
                    trace_log(LOG_INFO, "Black won by white Timeout");
                    return false;
                }
                if self.black_used_time >= black {
                    self.game_result = GameResult::BlackTimeout;
                    println!("{blue}White won by black Timeout{reset}");
                    trace_log(LOG_INFO, "White won by black Timeout");
                    return false;
                }

                return true;
            }
            _ => { true }
        }
    }
}