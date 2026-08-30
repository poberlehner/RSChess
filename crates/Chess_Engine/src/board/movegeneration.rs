use std::ops::Range;

use log::*;
use crate::board::Board;
use crate::board::r#move::{DIR_INDEX_TO_OFFSET, KNIGHT_JUMP_OFFSETS, Move};
use crate::board::piece::{Piece, PieceType};

#[allow(clippy::too_many_arguments)]
impl Board {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn add_move(&mut self, origin: u8, target: u8, promotion_type: PieceType, white: bool,
        pinned_orthogonal: bool, pinned_diagonal: bool, check: bool, moves: &mut Vec<Move>)
    {
        #[allow(clippy::unwrap_used, reason = "performance")]
        if check {
            if 1 << target & self.get_check_block_bitboard(white) == 0 {
                return;
            }
            if white {
                if self.white_double_check.unwrap() {
                    return;
                }
            }
            else if self.black_double_check.unwrap() {
                return;
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

    #[allow(clippy::collapsible_else_if)]
    ///Always returns a Some value
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
        
        if (self.white_double_check.is_none() && white) || (self.black_double_check.is_none() && !white) {
            error!("double check value is not set inside of get_legal_moves");
            panic!("Unset double check value");
        }

        for i in 0..64 {
            if let Some(piece) = self.squares[i] {

                #[allow(non_snake_case)]
                let I = i as i8;

                if piece.white_piece != white {
                    continue;
                }

                let pinned_orthogonal: bool = 1 << i & self.get_orthogonal_pin_bitboard(white) != 0;
                let pinned_diagonal: bool = 1 << i & self.get_diagonal_pin_bitboard(white) != 0;

                self.handle_piece_movegen(piece, &mut moves, i, I, white, pinned_orthogonal, pinned_diagonal, check);
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

    #[inline(always)]
    #[allow(non_snake_case)]
    fn handle_piece_movegen(&mut self, piece: Piece, moves: &mut Vec<Move>, i: usize, I: i8, white: bool, pinned_orthogonal: bool, pinned_diagonal: bool, check: bool) {
        match piece.piece_type {
            PieceType::King => {
                for dir in 0..8 {
                    if self.distance_to_edge[i][dir] == 0 {
                        continue;
                    }

                    if ((!(self.get_attacked_squares_bitboard(
                            !piece.white_piece) |
                            self.generate_pieces_bitboard(white))) &
                            (1 << (I + DIR_INDEX_TO_OFFSET[dir] as i8))) != 0
                        {
                            moves.push(Move { origin_square: i as u8, destination_square: (I + DIR_INDEX_TO_OFFSET[dir] as i8) as u8, promotion_type: PieceType::None});
                        }
                }
            }
            PieceType::Queen => {
                self.handle_directional_movegen(0..8, i, I, piece, white, pinned_diagonal, pinned_orthogonal, check, moves);
            }
            PieceType::Rook => {
                self.handle_directional_movegen(0..4, i, I, piece, white, pinned_diagonal, pinned_orthogonal, check, moves);
            }
            PieceType::Bishop => {
                self.handle_directional_movegen(4..8, i, I, piece, white, pinned_diagonal, pinned_orthogonal, check, moves);
            }
            PieceType::Knight => {
                for dir in 0..8 {
                    if !self.knight_jump_legality[i][dir] {
                        continue;
                    }

                    if let Some(target_piece) = self.squares[(I + KNIGHT_JUMP_OFFSETS[dir] as i8) as usize] {
                        if target_piece.white_piece != piece.white_piece {
                            self.add_move(i as u8, (I + KNIGHT_JUMP_OFFSETS[dir] as i8) as u8,
                            PieceType::None, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                        }
                    }
                    else {
                        self.add_move(i as u8, (I + KNIGHT_JUMP_OFFSETS[dir] as i8) as u8,
                        PieceType::None, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                    }
                }
            }
            PieceType::Pawn => {
                self.handle_pawn_movegen(i, I, piece, white, pinned_diagonal, pinned_orthogonal, check, moves);
            }
            _ => {
                error!("Piece type of square {} is set to None", i);
            }
        }
    }

    #[inline(always)]
    #[allow(non_snake_case)]
    fn handle_directional_movegen(&mut self, dirs: Range<usize>, i: usize, I: i8, piece: Piece,
        white: bool, pinned_diagonal: bool, pinned_orthogonal: bool, check: bool, moves: &mut Vec<Move>) {
        for dir in dirs {
            for n in 1..=self.distance_to_edge[i][dir] {
                let offset = DIR_INDEX_TO_OFFSET[dir] as i8 * n as i8;

                if let Some(target_piece) = self.squares[(I + offset) as usize] {
                    if target_piece.white_piece == piece.white_piece {
                        break;
                    }

                    self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                        PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, moves);

                    if target_piece.piece_type != PieceType::King {
                        break;
                    }
                }

                self.add_move(i as u8, (I as i32 + DIR_INDEX_TO_OFFSET[dir] * n as i32) as u8,
                    PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, moves);
            }
        }
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    fn handle_pawn_movegen(&mut self, i: usize, I: i8, piece: Piece, white: bool, pinned_diagonal: bool, pinned_orthogonal: bool, check: bool, moves: &mut Vec<Move>) {
        if i <= 7 || i > 55 {
             error!("A pawn managed to reach the last row \"{}\"", self.generate_fen());
         }

         let row_in_front = if piece.white_piece { i + 8 } else { i - 8 };

         if self.squares[row_in_front].is_none() {
             if row_in_front > 7 && row_in_front < 55 {
                 self.add_move(i as u8, row_in_front as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check,  moves);
             }

             if (piece.white_piece && i / 8 == 1) || (!piece.white_piece && i / 8 == 6) {
                 let row_2_in_front: usize = (row_in_front as i32 + if piece.white_piece { 8 } else { -8 }) as usize;

                 if self.squares[row_2_in_front].is_none() {
                     self.add_move(i as u8, row_2_in_front as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                 }
             }

             if row_in_front <= 7 || row_in_front > 55 {
                 self.add_move(i as u8, row_in_front as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                 self.add_move(i as u8, row_in_front as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                 self.add_move(i as u8, row_in_front as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                 self.add_move(i as u8, row_in_front as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check,  moves);
             }
         }

         if i % 8 != 7 {
             if let Some(piece_in_front) = self.squares[row_in_front + 1] {
                 if piece_in_front.white_piece != piece.white_piece {
                     if row_in_front <= 7 || row_in_front >= 55 {
                         self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                         self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                         self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                         self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, moves);
                     }
                     else {
                         self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check,  moves);
                     }
                 }
             }
             else {
                 if let Some(en_passant_square) = self.game_state.en_passant_square {
                    if  (en_passant_square as i8 - I == 9 && white) ||
                        (en_passant_square as i8 - I == -7 && !white) {
                        if 1 << i & self.get_en_passant_pin_bitboard(white) == 0 {
                            self.add_move(i as u8, (row_in_front + 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, moves);
                        }
                    }
                 }
             }
         }
         if !i.is_multiple_of(8) {
             if let Some(piece_in_front) = self.squares[row_in_front - 1] {
                 if piece_in_front.white_piece != white {
                     if row_in_front <= 7 || row_in_front > 55 {
                         self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check,moves);
                         self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, moves);
                         self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, moves);
                         self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, moves);
                     }
                     else {
                         self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, moves);
                     }
                 }
             }
             else {
                 if let Some(en_passant_square) = self.game_state.en_passant_square {
                    if (en_passant_square as i8 - I == 7 && self.game_state.white_to_move) ||
                        (en_passant_square as i8 - I == -9 && !self.game_state.white_to_move) {
                        if 1 << i & self.get_en_passant_pin_bitboard(white) == 0 {
                            self.add_move(i as u8, (row_in_front - 1) as u8, PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, moves);
                        }
                    }
                 }
             }
         }
    }

    pub fn get_legal_moves_for_square(&mut self, square: u8) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        #[allow(clippy::unwrap_used)]
        for m in self.get_legal_moves().as_ref().unwrap() {
            if m.origin_square == square {
                moves.push(*m);
            }
        }

        moves
    }
}