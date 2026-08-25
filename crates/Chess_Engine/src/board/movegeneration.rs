use log::*;
use crate::board::Board;
use crate::board::r#move::{DIR_INDEX_TO_OFFSET, KNIGHT_JUMP_OFFSETS, Move};
use crate::board::piece::PieceType;

impl Board {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn add_move(&mut self, origin: u8, target: u8, promotion_type: PieceType, white: bool,
        pinned_orthogonal: bool, pinned_diagonal: bool, check: bool, moves: &mut Vec<Move>) {
        if check {
            if 1 << target & self.get_check_block_bitboard(white) == 0 {
                return;
            }
            if white {
                if self.white_double_check.unwrap() {
                    return;
                }
            }
            else {
                if self.black_double_check.unwrap() {
                    return;
                }
            }
        }

        if pinned_orthogonal {
            //Only allow orthogonal movement
            if (((origin % 8).abs_diff(target % 8) != 0) ^ ((origin / 8).abs_diff(target / 8) != 0)) &&
                (1 << target & self.get_orthogonal_pin_bitboard(white) != 0) {
                moves.push(Move { origin_square: origin, destination_square: target, promotion_type });
            }
        }
        else if pinned_diagonal {
            //Only allow diagonal movement
            if ((origin % 8).abs_diff(target % 8) != 0) & ((origin / 8).abs_diff(target / 8) != 0) &&
                1 << target & self.get_diagonal_pin_bitboard(white) != 0 {
                moves.push( Move { origin_square: origin, destination_square: target, promotion_type });
            }
        }
        else {
            moves.push(Move { origin_square: origin, destination_square: target, promotion_type });
        }
    }

    pub fn get_legal_moves(&mut self) -> &Option<Vec<Move>> {
        if self.legal_moves.is_some() {
            return &self.legal_moves;
        }

        let white = self.game_state.white_to_move;

        let mut moves: Vec<Move> = Vec::with_capacity(30);

        let check: bool =
        if white {
             1 << self.white_king_pos & self.get_attacked_squares_bitboard(false) != 0
        } else {
            1 << self.black_king_pos & self.get_attacked_squares_bitboard(true) != 0
        };

        for i in 0..64 {
            if self.squares[i].is_some() {

                #[allow(non_snake_case)]
                let I = i as i8;

                let piece = self.squares[i].unwrap();

                if piece.white_piece != white {
                    continue;
                }

                let pinned_orthogonal: bool = 1 << i & self.get_orthogonal_pin_bitboard(white) != 0;
                let pinned_diagonal: bool = 1 << i & self.get_diagonal_pin_bitboard(white) != 0;

                match piece.piece_type {
                    PieceType::King => {
                        for dir in 0..8 {
                            if self.distance_to_edge[i][dir] == 0 {
                                continue;
                            }

                            if ((!(self.get_attacked_squares_bitboard(
                                    !self.squares[i].unwrap().white_piece) |
                                    self.generate_pieces_bitboard(white))) &
                                    (1 << (I + DIR_INDEX_TO_OFFSET[dir] as i8))) != 0
                                {
                                    moves.push(Move { origin_square: i as u8, destination_square: (I + DIR_INDEX_TO_OFFSET[dir] as i8) as u8, promotion_type: PieceType::None});
                                }
                        }
                    }
                    PieceType::Queen => {
                        for dir in 0..8 {
                            for n in 1..=self.distance_to_edge[i][dir] {
                                let offset = DIR_INDEX_TO_OFFSET[dir] as i8 * n as i8;

                                if self.squares[(I + offset) as usize].is_some() {
                                   if self.squares[(I + offset) as usize].unwrap().white_piece == piece.white_piece {
                                       break;
                                   }

                                   self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                       PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);

                                   if self.squares[(I + offset) as usize].unwrap().piece_type != PieceType::King {
                                       break;
                                   }
                                }

                                self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                    PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                            }
                        }
                    }
                    PieceType::Rook => {
                        for dir in 0..4 {
                            for n in 1..=self.distance_to_edge[i][dir] {
                                let offset = DIR_INDEX_TO_OFFSET[dir] as i8 * n as i8;
                                if self.squares[(I + offset) as usize].is_some() {
                                   if self.squares[(I + offset) as usize].unwrap().white_piece == piece.white_piece {
                                        break;
                                    }

                                    self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                        PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);

                                    if self.squares[(I + offset) as usize].unwrap().piece_type != PieceType::King {
                                        break;
                                    }
                                }

                                self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                    PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                            }
                        }
                    }
                    PieceType::Bishop => {
                        for dir in 4..8 {
                            for n in 1..=self.distance_to_edge[i][dir] {
                                let offset = DIR_INDEX_TO_OFFSET[dir] as i8 * n as i8;
                                if self.squares[(I + offset) as usize].is_some() {
                                    if self.squares[(I + offset) as usize].unwrap().white_piece == piece.white_piece {
                                        break;
                                     }

                                     self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                         PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);

                                     if self.squares[(I + offset) as usize].unwrap().piece_type != PieceType::King {
                                         break;
                                     }
                                 }

                                 self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                                     PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                             }
                         }
                    }
                    PieceType::Knight => {
                        for dir in 0..8 {
                            if !self.knight_jump_legality[i][dir] {
                                continue;
                            }

                            if self.squares[(I + KNIGHT_JUMP_OFFSETS[dir] as i8) as usize].is_some() {
                                if self.squares[(I + KNIGHT_JUMP_OFFSETS[dir] as i8) as usize].unwrap().white_piece != piece.white_piece {
                                    self.add_move(i as u8, (I + KNIGHT_JUMP_OFFSETS[dir] as i8) as u8,
                                    PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                }
                            }
                            else {
                                self.add_move(i as u8, (I + KNIGHT_JUMP_OFFSETS[dir] as i8) as u8,
                                PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                            }
                        }
                    }
                    PieceType::Pawn => {
                        if i <= 7 || i > 55 {
                            error!("A pawn managed to reach the last row \"{}\"", self.generate_fen());
                        }

                        let row_in_front = if piece.white_piece { i + 8 } else { i - 8 };

                        if self.squares[row_in_front].is_none() {
                            if row_in_front > 7 && row_in_front < 55 {
                                self.add_move(i as u8, row_in_front as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                            }

                            if (piece.white_piece && i / 8 == 1) || (!piece.white_piece && i / 8 == 6) {
                                let row_2_in_front: usize = (row_in_front as i32 + if piece.white_piece { 8 } else { -8 }) as usize;

                                if self.squares[row_2_in_front].is_none() {
                                    self.add_move(i as u8, row_2_in_front as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                }
                            }

                            //Promotions
                            if row_in_front <= 7 || row_in_front > 55 {
                                self.add_move(i as u8, row_in_front as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                self.add_move(i as u8, row_in_front as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                self.add_move(i as u8, row_in_front as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                self.add_move(i as u8, row_in_front as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                            }
                        }

                        if i % 8 != 7 {
                            if self.squares[row_in_front + 1].is_some() {
                                if self.squares[row_in_front + 1].unwrap().white_piece != self.squares[i].unwrap().white_piece {
                                    if row_in_front <= 7 || row_in_front >= 55 {
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                    else {
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                }
                            }
                            else {
                                if self.game_state.en_passant_square.is_some() &&
                                    ((self.game_state.en_passant_square.unwrap() as i8 - I == 9 && self.game_state.white_to_move) ||
                                    (self.game_state.en_passant_square.unwrap() as i8 - I == -7 && !self.game_state.white_to_move)) {
                                    if 1 << i & self.get_en_passant_pin_bitboard(white) == 0 {
                                        self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                }
                            }
                        }
                        if i % 8 != 0 {
                            if self.squares[row_in_front - 1].is_some() {
                                if self.squares[row_in_front - 1].unwrap().white_piece != self.squares[i].unwrap().white_piece {
                                    if row_in_front <= 7 || row_in_front > 55 {
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                    else {
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                }
                            }
                            else {
                                if self.game_state.en_passant_square.is_some() &&
                                    ((self.game_state.en_passant_square.unwrap() as i8 - I == 7 && self.game_state.white_to_move) ||
                                    (self.game_state.en_passant_square.unwrap() as i8 - I == -9 && !self.game_state.white_to_move)) {
                                    if 1 << i & self.get_en_passant_pin_bitboard(white) == 0 {
                                        self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        error!("Piece type of square {} is set to None", i);
                    }
                }
            }
        }

        if !self.check() {
            if self.game_state.castling_rights[0] && white {
                if self.squares[5].is_none() && self.squares[6].is_none() {
                    if ((1 << 5) | (1 << 6)) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: 4,
                            destination_square: 6,
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[1] && white {
                if self.squares[1].is_none() && self.squares[2].is_none() && self.squares[3].is_none() {
                    if ((1 << 1) | (1 << 2) | (1 << 3)) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: 4,
                            destination_square: 2,
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[2] && !white {
                if self.squares[61].is_none() && self.squares[62].is_none() {
                    if ((1 << 61) | (1 << 62)) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: 60,
                            destination_square: 62,
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[3] && !white {
                if self.squares[57].is_none() && self.squares[58].is_none() && self.squares[59].is_none() {
                    if ((1 << 57) | (1 << 58) | (1 << 59)) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: 60,
                            destination_square: 58,
                            promotion_type: PieceType::None,
                        })
                    }
                }
            }
        }

        self.legal_moves = Some(moves);

        &self.legal_moves
    }

    pub fn get_legal_moves_for_square(&mut self, square: u8) -> Vec<Move> {
        //Rewrite by simply filtering all moves
        let mut moves: Vec<Move> = vec![];

        for m in self.get_legal_moves().as_ref().unwrap().iter() {
            if m.origin_square == square {
                moves.push(*m);
            }
        }

        moves
    }
}