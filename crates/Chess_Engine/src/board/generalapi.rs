use chrono::{Datelike, Local, Timelike};
use log::*;
use crate::board::gamestate::GameResult;
use crate::board::piece::{Piece, PieceType};
use crate::board::r#move::{LogMove, Move};
use crate::board::Board;
use crate::board::utilities::TimeControl;
use std::cmp::max;
use std::time::*;
use std::io::prelude::*;

//Main api functions
impl Board {
    ///returns: success
    pub fn make_move(&mut self, m: Move, in_search: bool) -> bool {
        if self.game_result != GameResult::InProgress || ((in_search) &&
            (self.game_result == GameResult::Repetition || self.game_result == GameResult::FiftyMoveRule)) {
            println!("The game is not running");
            println!("Game Result: {:?}", self.game_result);
            warn!("Tried to make move {} in position \"{}\" while the game result is {:?}", m.to_string(), self.generate_fen(), self.game_result);

            self.move_log.push(LogMove::null_move());

            return false;
        }

        if self.legal_moves.is_none() {
            self.get_legal_moves();
        }

        if !self.legal_moves.as_ref().unwrap().contains(&m) {
            println!("Position: {}", self.generate_fen());
            println!("The Move: {} is not legal", m.to_string());

            warn!("Tried to make move {} in position \"{}\"", m.to_string(), self.generate_fen());

            self.move_log.push(LogMove::null_move());

            return false;
        }

        if !in_search {
            if !self.handle_time() {
                println!("No time left");
                info!("Tried to make move {} after timeout", m.to_string());

                self.move_log.push(LogMove::null_move());

                return false;
            }
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
            moved_piece: self.squares[m.origin_square as usize].unwrap(),
            null_move: false
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
                    error!("Promotion type of move {} is set to king in position \"{}\"", m.to_string(), self.generate_fen());
                    panic!("Invalid promotion type");
                }
                if self.squares[m.origin_square as usize]
                    .unwrap().piece_type != PieceType::Pawn {
                    error!("Promotion type of move {} is set even though the move originates from {:?}",
                            m.to_string(), self.squares[m.origin_square as usize].unwrap().piece_type);
                    panic!("invalid promotion type");
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

        self.invalidate_caches();

        self.move_log.push(current_logmove);

        self.game_state.white_to_move = !self.game_state.white_to_move;

        self.handle_end_conditions(in_search, &current_logmove);

        self.generate_all_bitboards();

        true
    }

    pub fn undo_move(&mut self, in_search: bool) -> Option<LogMove> {
        if self.move_log.is_empty() {
            warn!("Tried to undo move while the move-log is empty");
            return None;
        }

        if self.game_state.white_to_move {
            self.game_state.next_move_number = max(1, self.game_state.next_move_number as i64 - 1) as u32;
        }

        self.game_state.white_to_move = !self.game_state.white_to_move;

        let last = self.move_log.pop().unwrap();

        if last.null_move {
            return Some(last);
        }

        let piece = self.squares[last.destination_square as usize].expect("Invalid logged move");

        if !in_search {
            if piece.white_piece {
                self.white_time_left += last.time_taken;
            }
            else {
                self.black_time_left += last.time_taken;
            }
        }

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
            self.game_start_time = Local::now();

            self.game_result = GameResult::InProgress;

            self.time_control = time_control;
            match self.time_control {
                TimeControl::Limited(time, _) => {
                    self.white_time_left = time;
                    self.black_time_left = time;
                }
                TimeControl::Unlimited => {
                    self.white_time_left = Duration::from_hours(99999999);
                    self.black_time_left = Duration::from_hours(99999999);
                }
            }

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
            TimeControl::Limited(_, _) => {
                if white {
                    if self.game_state.white_to_move {
                        if self.move_start_time.elapsed() > self.white_time_left {
                            return Duration::new(0, 0);
                        }

                        return self.white_time_left - self.move_start_time.elapsed();
                    }
                    else {
                        return self.white_time_left;
                    }
                }
                else {
                    if self.game_state.white_to_move {
                        return self.black_time_left;
                    }
                    else {
                        if self.move_start_time.elapsed() > self.black_time_left {
                            return Duration::new(0, 0);
                        }

                        return self.black_time_left - self.move_start_time.elapsed();
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
            GameResult::BlackIsMated         => { 1 }
            GameResult::BlackTimeout         => { 1 }
            GameResult::WhiteIsMated         => { 2 }
            GameResult::WhiteTimeout         => { 2 }
            GameResult::InProgress           => {-1 }
            GameResult::NotStarted           => {-1 }
            GameResult::InsufficientMaterial => { 0 }
            GameResult::FiftyMoveRule        => { 0 }
            GameResult::Repetition           => { 0 }
            GameResult::Stalemate            => { 0 }
        }
    }

    pub fn get_move_history(&self) -> String {
        let mut result = "".to_string();
        let mut number: i32 = 1;

        let mut board = Board::new();
        board.load_fen(&board.start_fen.clone());
        board.start_game(TimeControl::Unlimited);

        for m in self.move_log.iter() {
            if number % 2 == 1 {
                result.push_str(&format!("{}. ", (number as f32 / 2.0).ceil()));
            }

            let move_str = board.log_move_to_string(m.clone());

            board.make_move(m.to_move(), false);

            if board.checkmate() {
                result.push_str(&move_str);
                result.push('#');
            }
            else if board.check() {
                result.push_str(&move_str);
                result.push('+');
            }
            else {
                result.push_str(&move_str);
            }

            number += 1;

            result.push(' ');
        }

        result.push_str(
            match self.get_game_result_simplified() {
            -1 => { "*" }
            0  => { "1/2-1/2" }
            1  => { "1-0" }
            2  => { "0-1" }
            _  => { "" }
        });

        result
    }

    ///Competitors are white then black
    pub fn get_pgn(&self, event: Option<&str>, competitors: Option<(&str, &str)>, round: Option<&str>, ) -> String {
        let mut result = "".to_string();

        result.push_str("[Event \"");
        result.push_str(&format!("{}", event.unwrap_or("-")));
        result.push_str("\"]\n");

        result.push_str("[Site \"RSChess\"]\n");

        result.push_str(&format!("[Date \"{}.{}.{}\"]\n",
            self.game_start_time.year(), self.game_start_time.month(), self.game_start_time.day()));
        result.push_str(&format!("[Time \"{}:{}:{}\"]\n",
            self.game_start_time.hour(), self.game_start_time.minute(), self.game_start_time.second()));

        result.push_str(&format!("[FEN \"{}\"]\n", self.start_fen));

        match self.time_control {
            TimeControl::Unlimited => { }
            TimeControl::Limited(time, increment) => {
                result.push_str(&format!("[TimeControl \"{}+{:.1}\"]\n", time.as_secs(), increment.as_secs_f32()));
            }
        }

        result.push_str(&format!("[Round \"{}\"]\n", round.unwrap_or("?")));

        result.push_str(&format!("[White \"{}\"]\n", competitors.unwrap_or(("-", "-")).0));
        result.push_str(&format!("[Black \"{}\"]\n", competitors.unwrap_or(("-", "-")).1));

        if self.get_game_result_simplified() != -1 {
            result.push_str("[Result \"");
            result.push_str(
                match self.get_game_result_simplified() {
                0  => { "1/2-1/2" }
                1  => { "1-0" }
                2  => { "0-1" }
                _  => { "game_result_simplified corrupted" }
            });
            result.push_str("\"]\n");
        }

        result.push('\n');

        result.push_str(&self.get_move_history());

        result
    }

    pub fn save_game(&mut self, save_file: String, event: Option<&str>, competitors: Option<(&str, &str)>, round: Option<&str>) {
        let pgn_file = std::fs::File::options().append(true).create(true).write(true).open(&save_file);

        if let Err(err) = pgn_file {
            warn!("Failed to open pgn save file: \"{}\" due to {}", &save_file, err);
            return;
        }

        let mut pgn_file = pgn_file.unwrap();
        let pgn = self.get_pgn(event, competitors, round);

        if let Err(err) = pgn_file.write(pgn.as_bytes()) {
            error!("Failed to write to pgn save file \"{}\" due to {}", &save_file, err);
            return;
        }

        if let Err(err) = pgn_file.write("\n\n".as_bytes()) {
            error!("Failed second write to pgn save file \"{}\" due to {}", &save_file, err);
        }
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

    ///Assumes the move hasn't been made yet
    fn log_move_to_string(&mut self, m: LogMove) -> String {
        if m.castling {
            if m.destination_square % 8 == 2 {
                return String::from("O-O-O");
            }
            else {
                return String::from("O-O");
            }
        }

        let mut str: String = String::new();

        self.test_duplicate_moves();

        let moves = self.get_legal_moves().as_ref().expect("Failed to get legal moves");

        let mut file_eq_count = 0;
        let mut rank_eq_count = 0;
        let mut pawn_eq_count = 0;
        let mut eq_count = 0;

        for m_local in moves.clone().iter() {
            let moved_piece = self.squares[m_local.origin_square as usize].clone().unwrap();
            if m_local.destination_square == m.destination_square {

                if moved_piece == m.moved_piece {
                    eq_count += 1;

                    if m_local.origin_square % 8 == m.origin_square % 8 {
                        file_eq_count += 1;
                    }
                    if m_local.origin_square / 8 == m.origin_square / 8 {
                        rank_eq_count += 1;
                    }
                }

                if m.moved_piece.piece_type == PieceType::Pawn && self.squares[m_local.origin_square as usize].unwrap().piece_type == PieceType::Pawn {
                    pawn_eq_count += 1;
                }
            }
        }

        if m.moved_piece.piece_type != PieceType::Pawn {
            str.push(m.moved_piece.to_char_uppercase());

            if rank_eq_count >= 2 || (file_eq_count < 2 && eq_count >= 2) {
                str.push(Self::file_index_to_char(m.origin_square as i32 % 8));
            }

            if file_eq_count >= 2 {
                str.push(char::from_digit(m.origin_square as u32 / 8 + 1, 10).unwrap());
            }
        }
        else {
            if pawn_eq_count >= 2 || (m.captured_piece.is_some() && m.captured_piece.unwrap().piece_type != PieceType::None) || m.en_passant {
                str.push(Self::file_index_to_char(m.origin_square as i32 % 8));
            }
        }

        if m.captured_piece.is_some() && m.captured_piece.unwrap().piece_type != PieceType::None  {
            str.push('x')
        }

        str.push_str(&format!("{}{}", Self::file_index_to_char(m.destination_square as i32 % 8), m.destination_square / 8 + 1));

        if m.promotion_type != PieceType::None {
            str.push('=');
            str.push(Self::piece_to_char(Piece { piece_type: m.promotion_type, white_piece: true }));
        }

        str
    }
}

///Handle End conditions
impl Board {
    #[allow(unused)]
    fn handle_end_conditions(&mut self, in_search: bool,  logmove: &LogMove) {
        let blue = "\x1b[34m";     // Blue text
        let reset = "\x1b[0m";     // Reset to default

        if self.checkmate() {
            if self.game_state.white_to_move {
                self.game_result = GameResult::WhiteIsMated;
                if !in_search {
                    println!("{blue}White Is Mated{reset}");
                    info!("White is mated");
                }

                return;
            }
            else {
                self.game_result = GameResult::BlackIsMated;
                if !in_search {
                    println!("{blue}Black Is Mated{reset}");
                    info!("Black is mated");
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
                    info!("Draw due to repetition");
                }
            }
        }

        if self.game_state.fifty_move_counter == 100 {
            self.game_result = GameResult::FiftyMoveRule;
            if !in_search {
                println!("{blue}Draw by the fifty-move rule{reset}");
                info!("Draw by the fifty-move rule");
            }
        }

        //Handle insufficient material
        if self.has_insufficient_material(true) && self.has_insufficient_material(false) {
            self.game_result = GameResult::InsufficientMaterial;
            if !in_search {
                println!("{blue}Draw due to insufficient material{reset}");
                info!("Draw due to insufficient material");
            }
        }

        //Handle Stalemate
        if self.get_legal_moves().as_ref().unwrap().is_empty() {
            self.game_result = GameResult::Stalemate;
            if !in_search {
                println!("{blue}Draw due to stalemate{reset}");
                info!("Draw due to stalemate");
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
            TimeControl::Limited(_, increment) => {
                let blue = "\x1b[34m";     // Blue text
                let reset = "\x1b[0m";     // Reset to default

                if self.game_state.white_to_move {
                    if self.white_time_left <= self.move_start_time.elapsed() {
                        self.white_time_left = Duration::new(0, 0);
                        self.game_result = GameResult::WhiteTimeout;
                        println!("{blue}Black won by white Timeout{reset}");
                        info!("Black won by white Timeout");
                        return false;
                    }

                    self.white_time_left -= self.move_start_time.elapsed();
                    self.white_time_left += increment;
                }
                else {
                    if self.black_time_left <= self.move_start_time.elapsed() {
                        self.black_time_left = Duration::new(0, 0);
                        self.game_result = GameResult::BlackTimeout;
                        println!("{blue}White won by black Timeout{reset}");
                        info!("White won by black Timeout");
                        return false;
                    }

                    self.black_time_left -= self.move_start_time.elapsed();
                    self.black_time_left += increment;
                }

                return true;
            }
            _ => { true }
        }
    }
}