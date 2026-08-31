use std::ops::Range;

use log::warn;

use crate::board::piece::{Piece, PieceType};
use crate::board::r#move::{DIR_INDEX_TO_OFFSET, KNIGHT_JUMP_OFFSETS, Move};
use crate::board::Board;
use crate::square_contains;

#[allow(clippy::too_many_arguments)]
impl Board {
    #[inline(always)]
    pub fn generate_pieces_bitboard(&mut self, white: bool) -> i64 {
        if white {
            if let Some(bitboard) = self.white_pieces_bitboard {
                return bitboard;
            }
        } else if let Some(bitboard) = self.black_pieces_bitboard {
            return bitboard;
        }

        let mut bitboard: i64 = 0;

        for (i, square) in self.squares.iter().enumerate() {
            if let Some(square) = square {
                bitboard |= ((square.white_piece == white) as i64) << i;
            }
        }

        if white {
            self.white_pieces_bitboard = Some(bitboard);
        } else {
            self.black_pieces_bitboard = Some(bitboard);
        }

        bitboard
    }


    #[inline(always)]
    pub fn get_all_pieces_bitboard(&mut self) -> i64 {
        self.generate_pieces_bitboard(false) | self.generate_pieces_bitboard(true)
    }

    #[inline(always)]
    pub fn get_color_to_move_pieces_bitboard(&mut self) -> i64 {
        self.generate_pieces_bitboard(self.game_state.white_to_move)
    }

    //Includes the movement of pinned pieces
    #[inline(always)]
    pub fn get_attacked_squares_bitboard(&mut self, white: bool) -> i64 {
        #[allow(clippy::unwrap_used)]
        if white {
            if self.white_attacked_squares_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_attacked_squares_bitboard.unwrap()
        } else {
            if self.black_attacked_squares_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_attacked_squares_bitboard.unwrap()
        }
    }

    #[inline(always)]
    pub fn get_orthogonal_pin_bitboard(&mut self, white: bool) -> i64 {
        #[allow(clippy::unwrap_used)]
        if white {
            if self.white_orthogonal_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_orthogonal_pin_bitboard.unwrap()
        } else {
            if self.black_orthogonal_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_orthogonal_pin_bitboard.unwrap()
        }
    }

    #[inline(always)]
    pub fn get_diagonal_pin_bitboard(&mut self, white: bool) -> i64 {
        #[allow(clippy::unwrap_used)]
        if white {
            if self.white_diagonal_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_diagonal_pin_bitboard.unwrap()
        } else {
            if self.black_diagonal_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_diagonal_pin_bitboard.unwrap()
        }
    }

    #[inline(always)]
    pub fn get_check_block_bitboard(&mut self, white: bool) -> i64 {
        if !self.check() {
            return 0;
        }

        #[allow(clippy::unwrap_used)]
        if white {
            if self.white_check_block_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_check_block_bitboard.unwrap()
        } else {
            if self.black_check_block_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_check_block_bitboard.unwrap()
        }
    }

    #[inline(always)]
    pub fn get_en_passant_pin_bitboard(&mut self, white: bool) -> i64 {
        #[allow(clippy::unwrap_used)]
        if white {
            if self.white_en_passant_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_en_passant_pin_bitboard.unwrap()
        } else {
            if self.black_en_passant_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_en_passant_pin_bitboard.unwrap()
        }
    }

    pub fn update_pieces_bitboards(&mut self, _m: Move) {
        todo!();
    }

    pub fn generate_all_bitboards(&mut self) {
        //The pieces bitboards are assumed to be up to date of None (the update_piece_bitboards function has to be called from make_move())

        if self.white_attacked_squares_bitboard.is_some() && self.black_attacked_squares_bitboard.is_some() &&
            self.white_en_passant_pin_bitboard.is_some() && self.black_en_passant_pin_bitboard.is_some() &&
            self.white_orthogonal_pin_bitboard.is_some() && self.black_orthogonal_pin_bitboard.is_some() &&
            self.white_diagonal_pin_bitboard.is_some() && self.black_diagonal_pin_bitboard.is_some() &&
            self.white_check_block_bitboard.is_some() && self.black_check_block_bitboard.is_some() {
            return;
        }

        let mut check_number: i32 = 0;

        let mut white_attack_bitboard: i64 = 0;
        let mut black_attack_bitboard: i64 = 0;
        let mut white_orthogonal_pin_bitboard: i64 = 0;
        let mut white_diagonal_pin_bitboard: i64 = 0;
        let mut black_orthogonal_pin_bitboard: i64 = 0;
        let mut black_diagonal_pin_bitboard: i64 = 0;
        let mut white_check_block_bitboard: i64 = 0;
        let mut black_check_block_bitboard: i64 = 0;
        let mut white_en_passant_pin_bitboard: i64 = 0;
        let mut black_en_passant_pin_bitboard: i64 = 0;

        for i in 0..64u8 {
            #[allow(non_snake_case)]
            let I = i as i32;

            let iu = i as usize;

            if let Some(p) = self.squares[iu] {
                self.handle_piece_bitboard_gen(p, i, I, &mut check_number,
                    &mut white_attack_bitboard, &mut black_attack_bitboard,
                    &mut white_check_block_bitboard, &mut black_check_block_bitboard,
                    &mut white_diagonal_pin_bitboard, &mut black_diagonal_pin_bitboard,
                    &mut white_orthogonal_pin_bitboard, &mut black_orthogonal_pin_bitboard,
                    &mut white_en_passant_pin_bitboard, &mut black_en_passant_pin_bitboard);
            }
        }

        if check_number >= 2 {
            if self.game_state.white_to_move {
                self.white_double_check = Some(true);
            }
            else {
                self.black_double_check = Some(true);
            }
        }
        else {
            if self.game_state.white_to_move {
                self.white_double_check = Some(false);
            }
            else {
                self.black_double_check = Some(false);
            }
        }

        self.white_check_block_bitboard = Some(white_check_block_bitboard);
        self.black_check_block_bitboard = Some(black_check_block_bitboard);
        self.white_attacked_squares_bitboard = Some(white_attack_bitboard);
        self.black_attacked_squares_bitboard = Some(black_attack_bitboard);
        self.white_orthogonal_pin_bitboard = Some(white_orthogonal_pin_bitboard);
        self.black_orthogonal_pin_bitboard = Some(black_orthogonal_pin_bitboard);
        self.white_diagonal_pin_bitboard = Some(white_diagonal_pin_bitboard);
        self.black_diagonal_pin_bitboard = Some(black_diagonal_pin_bitboard);
        self.white_en_passant_pin_bitboard = Some(white_en_passant_pin_bitboard);
        self.black_en_passant_pin_bitboard = Some(black_en_passant_pin_bitboard);
    }

    #[allow(non_snake_case, clippy::collapsible_else_if)]
    #[inline(always)]
    fn handle_piece_bitboard_gen(&self, p: Piece, i: u8, I: i32, check_number: &mut i32,
        white_attack_bitboard: &mut i64, black_attack_bitboard: &mut i64,
        white_check_block_bitboard: &mut i64, black_check_block_bitboard: &mut i64,
        white_diagonal_pin_bitboard: &mut i64, black_diagonal_pin_bitboard: &mut i64,
        white_orthogonal_pin_bitboard: &mut i64, black_orthogonal_pin_bitboard: &mut i64,
        white_en_passant_pin_bitboard: &mut i64, black_en_passant_pin_bitboard: &mut i64)
    {
        match p.piece_type {
            PieceType::King => {
                for dir in 0..8 {
                    if self.distance_to_edge[i as usize][dir] < 1 {
                        continue;
                    }

                    if p.white_piece {
                        *white_attack_bitboard |= 1 << (I + DIR_INDEX_TO_OFFSET[dir]);
                    }
                    else {
                        *black_attack_bitboard |= 1 << (I + DIR_INDEX_TO_OFFSET[dir]);
                    }
                }
            }
            PieceType::Queen => {
                self.handle_bitboard_gen_directional(
                    0..8, i, I, p, check_number,
                    white_attack_bitboard, black_attack_bitboard,
                    white_check_block_bitboard, black_check_block_bitboard,
                    white_diagonal_pin_bitboard, black_diagonal_pin_bitboard,
                    white_orthogonal_pin_bitboard, black_orthogonal_pin_bitboard,
                    white_en_passant_pin_bitboard, black_en_passant_pin_bitboard
                );
            }
            PieceType::Rook => {
                self.handle_bitboard_gen_directional(
                    0..4, i, I, p, check_number,
                    white_attack_bitboard, black_attack_bitboard,
                    white_check_block_bitboard, black_check_block_bitboard,
                    white_diagonal_pin_bitboard, black_diagonal_pin_bitboard,
                    white_orthogonal_pin_bitboard, black_orthogonal_pin_bitboard,
                    white_en_passant_pin_bitboard, black_en_passant_pin_bitboard
                );
            }
            PieceType::Bishop => {
                self.handle_bitboard_gen_directional(
                    4..8, i, I, p, check_number,
                    white_attack_bitboard, black_attack_bitboard,
                    white_check_block_bitboard, black_check_block_bitboard,
                    white_diagonal_pin_bitboard, black_diagonal_pin_bitboard,
                    white_orthogonal_pin_bitboard, black_orthogonal_pin_bitboard,
                    white_en_passant_pin_bitboard, black_en_passant_pin_bitboard
                );
            }
            PieceType::Knight => {
                for dir in 0..8 {
                    if !self.knight_jump_legality[i as usize][dir] {
                        continue;
                    }

                    if p.white_piece {
                        *white_attack_bitboard |= 1 << (I + KNIGHT_JUMP_OFFSETS[dir]);
                    } else {
                        *black_attack_bitboard |= 1 << (I + KNIGHT_JUMP_OFFSETS[dir]);
                    }

                    if self.squares[(I + KNIGHT_JUMP_OFFSETS[dir]) as usize].is_some() {
                        if square_contains!(self, I + KNIGHT_JUMP_OFFSETS[dir], !p.white_piece, PieceType::King) {
                            *check_number += 1;

                            if p.white_piece {
                                *black_check_block_bitboard |= (1 << (I + KNIGHT_JUMP_OFFSETS[dir])) | (1 << i);
                            } else {
                                *white_check_block_bitboard |= (1 << (I + KNIGHT_JUMP_OFFSETS[dir])) | (1 << i);
                            }
                        }
                    }
                }
            }
            PieceType::Pawn => {
                self.handle_pawn_bitboard_gen(
                    i, p,
                    white_attack_bitboard, black_attack_bitboard,
                    white_check_block_bitboard, black_check_block_bitboard
                );
            }
            _ => {
                warn!("PieceType on square {i} or {} set to None", Self::square_index_to_string(i));
            }
        }
    }

    #[allow(non_snake_case)]
    #[allow(clippy::cognitive_complexity)]
    #[inline(always)]
    fn handle_bitboard_gen_directional(&self, dirs: Range<usize>, i: u8, I: i32, p: Piece, check_number: &mut i32,
        white_attack_bitboard: &mut i64, black_attack_bitboard: &mut i64,
        white_check_block_bitboard: &mut i64, black_check_block_bitboard: &mut i64,
        white_diagonal_pin_bitboard: &mut i64, black_diagonal_pin_bitboard: &mut i64,
        white_orthogonal_pin_bitboard: &mut i64, black_orthogonal_pin_bitboard: &mut i64,
        white_en_passant_pin_bitboard: &mut i64, black_en_passant_pin_bitboard: &mut i64)
    {
        for dir in dirs {
            let mut potential_bitboard: i64 = 0;
            let mut offset = DIR_INDEX_TO_OFFSET[dir];
            let mut pin_detection_stage: i32 = 0;

            //1 = a piece of different color is hit; -1 = not possible in the first place; -2 = not possible anymore
            let mut en_passant_pin_stage = 0;
            //Stage: 1: (only possible on non-diagonal movement) Pawn hit that can potentially:
            //          1) take another pawn by en passant
            //          2) be taken by en passant
            //Stage 1 task: search for a second pawn immedeately after the first one
            //Advance to Stage 2 if: a pawn is hit right after the first one that can:
            //  in Case 1) be taken by en passant by the first pawn
            //  in Case 2) take the first one by en passant
            //Stage 2 task: serach for the king of the color of the pawn to take the other one by en passant
            //Stage -1: Not possible anymore
            let mut first_hit_white: bool = false;
            let mut first_pawn_double_moved: bool = false;

            if self.game_state.en_passant_square.is_none() {
                en_passant_pin_stage = -1;
            }
            if DIR_INDEX_TO_OFFSET[dir] / 8 != 0 {
                en_passant_pin_stage = -1;
            }

            for _ in 0..self.distance_to_edge[i as usize][dir] {
                if I + offset < 0 || I + offset >= 64 {
                    if pin_detection_stage == 0 && (en_passant_pin_stage == 0 || en_passant_pin_stage == -1) {
                        if p.white_piece {
                            *white_attack_bitboard |= potential_bitboard;
                        } else {
                            *black_attack_bitboard |= potential_bitboard;
                        }
                    }
                    break;
                }

                potential_bitboard |= 1 << (I + offset);

                if pin_detection_stage == 0 {
                    if p.white_piece {
                        *white_attack_bitboard |= potential_bitboard;
                    } else {
                        *black_attack_bitboard |= potential_bitboard;
                    }
                }

                if pin_detection_stage != -1 {
                    if let Some(piece) = self.squares[(I + offset) as usize] {
                        if !square_contains!(self, I + offset, !p.white_piece, PieceType::King) {
                            if pin_detection_stage == 1 || piece.white_piece == p.white_piece {
                                pin_detection_stage = -1;
                            } else {
                                pin_detection_stage = 1;
                            }
                        } else {
                            if pin_detection_stage == 1 {
                                if (DIR_INDEX_TO_OFFSET[dir] % 8).abs() + (DIR_INDEX_TO_OFFSET[dir] / 8).abs() == 1 {
                                    if p.white_piece {
                                        *black_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << (I + offset));
                                    } else {
                                        *white_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << (I + offset));
                                    }
                                } else {
                                    if p.white_piece {
                                        *black_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << (I + offset));
                                    } else {
                                        *white_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << (I + offset));
                                    }
                                }

                                pin_detection_stage = -1;
                            } else {
                                *check_number += 1;

                                if p.white_piece {
                                    *black_check_block_bitboard |= potential_bitboard | 1 << i;
                                } else {
                                    *white_check_block_bitboard |= potential_bitboard | 1 << i;
                                }
                            }
                        }
                    }
                }

                if en_passant_pin_stage == 0 {
                    if let Some(piece) = self.squares[(I + offset) as usize] {
                        if piece.piece_type == PieceType::Pawn {
                            //If the pawn can be taken by en passant or can take another one by en passant
                            #[allow(clippy::unwrap_used, reason = "Checked")]
                            if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8) as u8 {
                                first_hit_white = piece.white_piece;
                                first_pawn_double_moved = true;

                                en_passant_pin_stage = 1;

                                offset += DIR_INDEX_TO_OFFSET[dir];

                                continue;
                            } else if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8 + DIR_INDEX_TO_OFFSET[dir] % 8) as u8 {
                                first_hit_white = piece.white_piece;
                                first_pawn_double_moved = false;

                                en_passant_pin_stage = 1;

                                offset += DIR_INDEX_TO_OFFSET[dir];

                                continue;
                            } else {
                                en_passant_pin_stage = -2;
                            }
                        } else {
                            en_passant_pin_stage = -2;
                        }
                    }
                }
                #[allow(clippy::unwrap_used)]
                if en_passant_pin_stage == 1 {
                    //If a second Pawn is hit that is the opposite color
                    if self.squares[(I + offset) as usize].is_some() && self.squares[(I + offset) as usize].unwrap().piece_type ==
                        PieceType::Pawn && self.squares[(I + offset) as usize].unwrap().white_piece == !first_hit_white {
                        if !first_pawn_double_moved {
                            if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8) as u8 {
                                en_passant_pin_stage = 2;

                                offset += DIR_INDEX_TO_OFFSET[dir];

                                continue;
                            } else {
                                en_passant_pin_stage = -2;
                            }
                        } else {
                            en_passant_pin_stage = 2;

                            offset += DIR_INDEX_TO_OFFSET[dir];

                            continue;
                        }
                    }
                }
                if en_passant_pin_stage == 2 {
                    if let Some(piece) = self.squares[(I + offset) as usize] {
                        if piece.piece_type == PieceType::King {
                            //if self.squares[(I + offset) as usize].unwrap().white_piece != p.white_piece {
                            if p.white_piece {
                                *black_en_passant_pin_bitboard |= potential_bitboard;
                            } else {
                                *white_en_passant_pin_bitboard |= potential_bitboard;
                            }
                            // }
                            // else {
                            //     en_passant_pin_stage = -2;
                            // }
                        } else {
                            en_passant_pin_stage = -2;
                        }
                    }
                }

                if pin_detection_stage == -1 && (en_passant_pin_stage == -1 || en_passant_pin_stage == -2) {
                    break;
                }

                offset += DIR_INDEX_TO_OFFSET[dir];
            }
        }
    }

    #[inline(always)]
    fn handle_pawn_bitboard_gen(&self, i: u8, p: Piece,
        white_attack_bitboard: &mut i64, black_attack_bitboard: &mut i64,
        white_check_block_bitboard: &mut i64, black_check_block_bitboard: &mut i64) {
        if !(8..=55).contains(&i) {
            warn!("A pawn is on the last rank");
            return;
        }

        let row_in_front: u8 = if p.white_piece { i + 8 } else { i - 8 };

        if !i.is_multiple_of(8) {
            if p.white_piece {
                *white_attack_bitboard |= 1 << (row_in_front - 1);
            } else {
                *black_attack_bitboard |= 1 << (row_in_front - 1);
            }

            if square_contains!(self, row_in_front - 1, !p.white_piece, PieceType::King) {
                if p.white_piece {
                    *black_check_block_bitboard |= (1 << (row_in_front - 1)) | (1 << i);
                } else {
                    *white_check_block_bitboard |= (1 << (row_in_front - 1)) | (1 << i);
                }
            }
        }
        if i % 8 != 7 {
            if p.white_piece {
                *white_attack_bitboard |= 1 << (row_in_front + 1);
            } else {
                *black_attack_bitboard |= 1 << (row_in_front + 1);
            }

            if square_contains!(self, row_in_front + 1, !p.white_piece, PieceType::King) {
                if p.white_piece {
                    *black_check_block_bitboard |= (1 << (row_in_front + 1)) | (1 << i);
                } else {
                    *white_check_block_bitboard |= (1 << (row_in_front + 1)) | (1 << i);
                }
            }
        }
    }
}