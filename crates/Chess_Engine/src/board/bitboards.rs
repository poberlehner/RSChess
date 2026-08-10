use crate::board::piece::PieceType;
use crate::board::r#move::{DIR_INDEX_TO_OFFSET, KNIGHT_JUMP_OFFSETS, Move};
use crate::board::Board;
use crate::square_contains;

impl Board {
    #[inline(always)]
    pub fn generate_pieces_bitboard(&mut self, white: bool) -> i64 {
        if white {
            if self.white_pieces_bitboard.is_some() {
                return self.white_pieces_bitboard.unwrap();
            }
        } else {
            if self.black_pieces_bitboard.is_some() {
                return self.black_pieces_bitboard.unwrap();
            }
        }

        let mut bitboard: i64 = 0;

        for i in 0..64 {
            if self.squares[i].is_some() {
                bitboard = bitboard | ((self.squares[i].unwrap().white_piece == white) as i64) << i;
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

        // let mut bitboard: i64 = 0;
        //
        // for y in 0i32..8 {
        //     for x in 0i32..8 {
        //         if self.squares[x as usize][y as usize].is_none() {
        //             continue;
        //         }
        //
        //         if self.squares[x as usize][y as usize].unwrap().white_piece != white {
        //             continue;
        //         }
        //
        //         match self.squares[x as usize][y as usize].unwrap().piece_type {
        //             PieceType::King => {
        //                 for i in (-1i32 + (x == 0) as i32)..(2 - (x == 7) as i32) {
        //                     for j in (-1i32 + (y == 0) as i32)..(2 - (y == 7) as i32) {
        //                         if i != 0 || j != 0 {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + i) as u8, (y + j) as u8));
        //                         }
        //                     }
        //                 }
        //             }
        //             PieceType::Pawn => {
        //                 if y == 0 || y == 7 {
        //                     continue;
        //                 }
        //
        //                 let row_in_front = y as usize - white as usize + !white as usize;
        //
        //                 if x != 7 {
        //                     bitboard = bitboard | 1 << Self::index_from_coord((x as u8 + 1, row_in_front as u8));
        //                 }
        //                 if x != 0 {
        //                     bitboard = bitboard | 1 << Self::index_from_coord((x as u8 - 1, row_in_front as u8));
        //                 }
        //             }
        //             PieceType::Queen => {
        //                 for direction in [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
        //                     let mut check_offset = (0, 0);
        //                     let mut obstructed: bool = false;
        //
        //                     if check_offset.0 + direction.0 + x < 0 || check_offset.0 + direction.0 + x > 7 {
        //                         obstructed = true;
        //                     }
        //                     if check_offset.1 + direction.1 + y < 0 || check_offset.1 + direction.1 + y > 7 {
        //                         obstructed = true;
        //                     }
        //
        //                     while !obstructed {
        //                         check_offset.0 += direction.0;
        //                         check_offset.1 += direction.1;
        //
        //                         if self.squares[(x + check_offset.0) as usize][(y + check_offset.1) as usize].is_none() {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //                         }
        //                         else {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //
        //                             //don't break when the enemy king is blocking to prevent him from moving back
        //                             if !square_contains!(self, x + check_offset.0, y + check_offset.1, !white, PieceType::King) {
        //                                 break;
        //                             }
        //                         }
        //
        //                         if x + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                         if x + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                         if y + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                         if y + check_offset.1 == 0 && direction.1 == -1 { break; }
        //                     }
        //                 }
        //             }
        //             PieceType::Rook => {
        //                 for direction in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
        //                     let mut check_offset = (0, 0);
        //                     let mut obstructed: bool = false;
        //
        //                     if check_offset.0 + direction.0 + x < 0 || check_offset.0 + direction.0 + x > 7 {
        //                         obstructed = true;
        //                     }
        //                     if check_offset.1 + direction.1 + y < 0 || check_offset.1 + direction.1 + y > 7 {
        //                         obstructed = true;
        //                     }
        //
        //                     while !obstructed {
        //                         check_offset.0 += direction.0;
        //                         check_offset.1 += direction.1;
        //
        //                         if self.squares[(x + check_offset.0) as usize][(y + check_offset.1) as usize].is_none() {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //                         }
        //                         else {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //
        //                             //don't break when the enemy king is blocking to prevent him from moving back
        //                             if !square_contains!(self, x + check_offset.0, y + check_offset.1, !white, PieceType::King) {
        //                                 break;
        //                             }
        //                         }
        //
        //                         if x + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                         if x + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                         if y + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                         if y + check_offset.1 == 0 && direction.1 == -1 { break; }
        //                     }
        //                 }
        //             }
        //             PieceType::Bishop => {
        //                 for direction in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
        //                     let mut check_offset = (0, 0);
        //                     let mut obstructed: bool = false;
        //
        //                     if check_offset.0 + direction.0 + x < 0 || check_offset.0 + direction.0 + x > 7 {
        //                         obstructed = true;
        //                     }
        //                     if check_offset.1 + direction.1 + y < 0 || check_offset.1 + direction.1 + y > 7 {
        //                         obstructed = true;
        //                     }
        //
        //                     while !obstructed {
        //                         check_offset.0 += direction.0;
        //                         check_offset.1 += direction.1;
        //
        //                         if self.squares[(x + check_offset.0) as usize][(y + check_offset.1) as usize].is_none() {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //                         } else {
        //                             bitboard = bitboard | 1 << Self::index_from_coord(((x + check_offset.0) as u8, (y + check_offset.1) as u8));
        //
        //                             //don't break when the enemy king is the blocking piece to prevent him from moving back (makes it easier for eval algorithms to evaluate skewers correctly too)
        //                             if !square_contains!(self, x + check_offset.0, y + check_offset.1, !white, PieceType::King) {
        //                                 break;
        //                             }
        //                         }
        //
        //                         if x + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                         if x + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                         if y + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                         if y + check_offset.1 == 0 && direction.1 == -1 { break; }
        //                     }
        //                 }
        //             }
        //             PieceType::Knight => {
        //                 for i in [-2i32, -1, 1, 2] {
        //                     for j in [-2i32, -1, 1, 2] {
        //                         if i.abs() != j.abs() {
        //                             if x + i <= 7 && x + i >= 0 && y + j <= 7 && y + j >= 0 {
        //                                 bitboard = bitboard | 1 << Self::index_from_coord(((x + i) as u8, (y + j) as u8));
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //             _ => {
        //                 eprintln!("Piece on square ({}, {}) set to None", x,y);
        //             }
        //         }
        //     }
        // }
        //
        // if white {
        //     self.white_attacked_squares_bitboard = Some(bitboard);
        // }
        // else {
        //     self.black_attacked_squares_bitboard = Some(bitboard);
        // }
        //
        // bitboard
    }

    #[inline(always)]
    pub fn get_orthogonal_pin_bitboard(&mut self, white: bool) -> i64 {
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

        // let mut king_square: (i8, i8) = (11, 11);
        //
        // for x in 0..8 {
        //     for y in 0..8 {
        //         if self.squares[iu].is_some() && self.squares[iu].unwrap().piece_type == PieceType::King && self.squares[iu].unwrap().white_piece == white {
        //             king_square = (x as i8, y as i8);
        //         }
        //     }
        // }
        //
        // if king_square.0 == 11 || king_square.1 == 11 {
        //     println!("Unable to find king square!");
        // }
        //
        // let mut bitboard = 0;
        //
        // for direction in [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
        //     let mut potential_bitboard: i64 = 0;
        //     let mut check_offset = direction;
        //     let mut potential_pin = false;
        //
        //     loop {
        //         if  (king_square.0 + check_offset.0) > 7 || (king_square.1 + check_offset.1) > 7 ||
        //             (king_square.0 + check_offset.0) < 0 || (king_square.1 + check_offset.1) < 0 {
        //             break;
        //         }
        //
        //         if check_offset.0 != 0 && check_offset.1 != 0 {
        //             potential_bitboard |= 1 << Self::index_from_coord(((king_square.0 + check_offset.0) as u8, (king_square.1 + check_offset.1) as u8));
        //         }
        //
        //         if self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].is_some() {
        //             if potential_pin {
        //                 //break, if another piece of the kings color comes after
        //                 if self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].unwrap().white_piece == white {
        //                     break;
        //                 }
        //
        //                 //the piece has to be a diagonally sliding piece to pin
        //                 if direction.0 != 0 && direction.1 != 0 {
        //                     if  self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].unwrap().piece_type == PieceType::Bishop ||
        //                         self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].unwrap().piece_type == PieceType::Queen {
        //                         bitboard |= potential_bitboard;
        //                     }
        //                 }
        //                 //the piece has to be laterally sliding
        //                 else {
        //                     if  self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].unwrap().piece_type == PieceType::Rook ||
        //                         self.squares[(check_offset.0 + king_square.0) as usize][(check_offset.1 + king_square.1) as usize].unwrap().piece_type == PieceType::Queen {
        //                         bitboard |= potential_bitboard;
        //                     }
        //                 }
        //             }
        //             else {
        //                 if self.squares[(king_square.0 + check_offset.0) as usize][(king_square.1 + check_offset.1) as usize].unwrap().white_piece == white {
        //                     potential_pin = true;
        //                 }
        //             }
        //         }
        //
        //
        //         check_offset.0 += direction.0;
        //         check_offset.1 += direction.1;
        //     }
        // }
        //
        // if white {
        //     self.white_pin_king_bitboard = Some(bitboard);
        // }
        // else {
        //     self.black_pin_king_bitboard = Some(bitboard);
        // }
        //
        // bitboard
    }

    #[inline(always)]
    pub fn get_diagonal_pin_bitboard(&mut self, white: bool) -> i64 {
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
        if self.check() == false {
            return 0;
        }

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

        // let king_square = if white { self.white_king_pos } else { self.black_king_pos };
        // let king_square = (king_square.0 as i8, king_square.1 as i8);
        //
        // let mut bitboard: i64 = 0;
        //
        // for direction in [(0, 1), (0, -1), (1, 0), (-1, 0), (-1, 1), (-1, -1), (1, -1), (1, 1)] {
        //     let mut check_offset = direction;
        //     let mut potential_bitboard: i64 = 1 << Self::index_from_coord((king_square.0 as u8, king_square.1 as u8));
        //
        //     loop {
        //         if ((check_offset.0 + king_square.0) < 0) || ((check_offset.1 + king_square.1) < 0) {
        //             break;
        //         }
        //         if ((check_offset.0 + king_square.0) > 7) || ((check_offset.1 + king_square.1) > 7) {
        //             break;
        //         }
        //
        //         potential_bitboard |= 1 << Self::index_from_coord(((king_square.0 + check_offset.0) as u8, (king_square.1 + check_offset.1) as u8));
        //
        //         if  self.squares[(king_square.0 + check_offset.0) as usize][(king_square.1 + check_offset.1) as usize].is_some() ||
        //             square_contains!(self, (king_square.0 + check_offset.0), king_square.1 + check_offset.1, white, PieceType::King) {
        //             if direction.0 != 0 && direction.1 != 0 {
        //                 if square_contains!(self, king_square.0 + check_offset.0, king_square.1 + check_offset.1, !white, PieceType::Bishop) || square_contains!(self, king_square.0 + check_offset.0, king_square.1 + check_offset.1, !white, PieceType::Queen) {
        //                     bitboard |= potential_bitboard;
        //                 }
        //                 if square_contains!(self, king_square.0 + check_offset.0, king_square.1 + check_offset.1, !white, PieceType::Pawn) && (check_offset.0.abs() == 1 && check_offset.1.abs() == 1) {
        //                     bitboard |= potential_bitboard;
        //                 }
        //             }
        //             else {
        //                 if square_contains!(self, king_square.0 + check_offset.0, king_square.1 + check_offset.1, !white, PieceType::Rook)  || square_contains!(self, king_square.0 + check_offset.0, king_square.1 + check_offset.1, !white, PieceType::Queen) {
        //                     bitboard |= potential_bitboard;
        //                 }
        //             }
        //
        //             break;
        //         }
        //
        //         check_offset.0 += direction.0;
        //         check_offset.1 += direction.1;
        //     }
        // }
        //
        // for offset in [(-1, -2), (-2, -1), (-2, 1), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)] {
        //     if  offset.0 + king_square.0 < 0 || offset.1 + king_square.1 < 0 ||
        //         offset.0 + king_square.0 > 7 || offset.1 + king_square.1 > 7 {
        //         continue;
        //     }
        //
        //     if square_contains!(self, offset.0 + king_square.0, offset.1 + king_square.1, !white, PieceType::Knight) {
        //         bitboard |= 1 << Self::index_from_coord((king_square.0 as u8, king_square.1 as u8));
        //         bitboard |= 1 << Self::index_from_coord(((offset.0 + king_square.0) as u8, (offset.1 + king_square.1) as u8));
        //     }
        // }
        //
        // if white {
        //     self.white_check_block_bitboard = Some(bitboard);
        // }
        // else {
        //     self.black_check_block_bitboard = Some(bitboard);
        // }
        //
        // bitboard
    }

    #[inline(always)]
    pub fn get_en_passant_pin_bitboard(&mut self, white: bool) -> i64 {
        if white {
            if self.white_en_passant_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.white_en_passant_pin_bitboard.expect("generate_all_bitboards didn't generate white_en_passant_pin_bitboard")
        } else {
            if self.black_en_passant_pin_bitboard.is_none() {
                self.generate_all_bitboards();
            }

            self.black_en_passant_pin_bitboard.expect("generate_all_bitboards didn't generate black_en_passant_pin_bitboard")
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
            //println!("Early return");
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

            if self.squares[iu].is_some() {
                let p = self.squares[iu].unwrap();

                match p.piece_type {
                    PieceType::King => {
                        for dir in 0..8 {
                            if self.distance_to_edge[i as usize][dir] < 1 {
                                continue;
                            }

                            if p.white_piece {
                                white_attack_bitboard |= 1 << I + DIR_INDEX_TO_OFFSET[dir];
                            }
                            else {
                                black_attack_bitboard |= 1 << I + DIR_INDEX_TO_OFFSET[dir];
                            }
                        }
                    }
                    PieceType::Queen => {
                        for dir in 0..8 {
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
                                            white_attack_bitboard |= potential_bitboard;
                                        } else {
                                            black_attack_bitboard |= potential_bitboard;
                                        }
                                    }
                                    break;
                                }

                                potential_bitboard |= 1 << I + offset;

                                if pin_detection_stage == 0 {
                                    if p.white_piece {
                                        white_attack_bitboard |= potential_bitboard;
                                    } else {
                                        black_attack_bitboard |= potential_bitboard;
                                    }
                                }

                                if pin_detection_stage != -1 {
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if !square_contains!(self, I + offset, !p.white_piece, PieceType::King) {
                                            if pin_detection_stage == 1 {
                                                pin_detection_stage = -1;
                                            } else {
                                                if self.squares[(I + offset) as usize].unwrap().white_piece == p.white_piece {
                                                    pin_detection_stage = -1;
                                                } else {
                                                    pin_detection_stage = 1;
                                                }
                                            }
                                        } else {
                                            if pin_detection_stage == 1 {
                                                if (DIR_INDEX_TO_OFFSET[dir] % 8).abs() + (DIR_INDEX_TO_OFFSET[dir] / 8).abs() == 1 {
                                                    if p.white_piece {
                                                        black_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    } else {
                                                        white_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    }
                                                } else {
                                                    if p.white_piece {
                                                        black_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    } else {
                                                        white_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    }
                                                }

                                                pin_detection_stage = -1;
                                            } else {
                                                check_number += 1;

                                                if p.white_piece {
                                                    black_check_block_bitboard |= potential_bitboard | 1 << i;
                                                } else {
                                                    white_check_block_bitboard |= potential_bitboard | 1 << i;
                                                }
                                            }
                                        }
                                    }
                                }

                                if en_passant_pin_stage == 0 {
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if self.squares[(I + offset) as usize].unwrap().piece_type == PieceType::Pawn {
                                            //If the pawn can be taken by en passant or can take another one by en passant
                                            if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8) as u8 {
                                                first_hit_white = self.squares[(I + offset) as usize].unwrap().white_piece;
                                                first_pawn_double_moved = true;

                                                en_passant_pin_stage = 1;

                                                offset += DIR_INDEX_TO_OFFSET[dir];

                                                continue;
                                            } else if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8 + DIR_INDEX_TO_OFFSET[dir] % 8) as u8 {
                                                first_hit_white = self.squares[(I + offset) as usize].unwrap().white_piece;
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
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if self.squares[(I + offset) as usize].unwrap().piece_type == PieceType::King {
                                            //if self.squares[(I + offset) as usize].unwrap().white_piece != p.white_piece {
                                            if p.white_piece {
                                                black_en_passant_pin_bitboard |= potential_bitboard;
                                            } else {
                                                white_en_passant_pin_bitboard |= potential_bitboard;
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
                    PieceType::Rook => {
                        for dir in 0..4 {
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
                                            white_attack_bitboard |= potential_bitboard;
                                        } else {
                                            black_attack_bitboard |= potential_bitboard;
                                        }
                                    }
                                    break;
                                }

                                potential_bitboard |= 1 << I + offset;

                                if pin_detection_stage == 0 {
                                    if p.white_piece {
                                        white_attack_bitboard |= potential_bitboard;
                                    } else {
                                        black_attack_bitboard |= potential_bitboard;
                                    }
                                }

                                if pin_detection_stage != -1 {
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if !square_contains!(self, I + offset, !p.white_piece, PieceType::King) {
                                            if pin_detection_stage == 1 {
                                                pin_detection_stage = -1;
                                            } else {
                                                if self.squares[(I + offset) as usize].unwrap().white_piece == p.white_piece {
                                                    pin_detection_stage = -1;
                                                } else {
                                                    pin_detection_stage = 1;
                                                }
                                            }
                                        } else {
                                            if pin_detection_stage == 1 {
                                                if (DIR_INDEX_TO_OFFSET[dir] % 8).abs() + (DIR_INDEX_TO_OFFSET[dir] / 8).abs() == 1 {
                                                    if p.white_piece {
                                                        black_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    } else {
                                                        white_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    }
                                                } else {
                                                    if p.white_piece {
                                                        black_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    } else {
                                                        white_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                    }
                                                }

                                                pin_detection_stage = -1;
                                            } else {
                                                check_number += 1;

                                                if p.white_piece {
                                                    black_check_block_bitboard |= potential_bitboard | 1 << i;
                                                } else {
                                                    white_check_block_bitboard |= potential_bitboard | 1 << i;
                                                }
                                            }
                                        }
                                    }
                                }

                                if en_passant_pin_stage == 0 {
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if self.squares[(I + offset) as usize].unwrap().piece_type == PieceType::Pawn {
                                            //If the pawn can be taken by en passant or can take another one by en passant
                                            if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8) as u8 {
                                                first_hit_white = self.squares[(I + offset) as usize].unwrap().white_piece;
                                                first_pawn_double_moved = true;

                                                en_passant_pin_stage = 1;

                                                offset += DIR_INDEX_TO_OFFSET[dir];

                                                continue;
                                            } else if self.game_state.en_passant_square.unwrap() % 8 == (I % 8 + offset % 8 + DIR_INDEX_TO_OFFSET[dir] % 8) as u8 {
                                                first_hit_white = self.squares[(I + offset) as usize].unwrap().white_piece;
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
                                    if self.squares[(I + offset) as usize].is_some() {
                                        if self.squares[(I + offset) as usize].unwrap().piece_type == PieceType::King {
                                            //if self.squares[(I + offset) as usize].unwrap().white_piece != p.white_piece {
                                            if p.white_piece {
                                                black_en_passant_pin_bitboard |= potential_bitboard;
                                            } else {
                                                white_en_passant_pin_bitboard |= potential_bitboard;
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
                    PieceType::Bishop => {
                        for dir in 4..8 {
                            let mut potential_bitboard: i64 = 0;
                            let mut offset = DIR_INDEX_TO_OFFSET[dir];
                            let mut pin_detection = false;

                            for _ in 0..self.distance_to_edge[i as usize][dir] {
                                potential_bitboard |= 1 << I + offset;

                                if self.squares[(I + offset) as usize].is_some() {
                                    if !square_contains!(self, I + offset, !p.white_piece, PieceType::King) {
                                        if pin_detection {
                                            break;
                                        }

                                        if p.white_piece {
                                            white_attack_bitboard |= potential_bitboard;
                                        } else {
                                            black_attack_bitboard |= potential_bitboard;
                                        }

                                        if self.squares[(I + offset) as usize].unwrap().white_piece == p.white_piece {
                                            break;
                                        } else {
                                            pin_detection = true;
                                        }
                                    } else {
                                        if pin_detection {
                                            if (DIR_INDEX_TO_OFFSET[dir] % 8).abs() + (DIR_INDEX_TO_OFFSET[dir] / 8).abs() == 1 {
                                                if p.white_piece {
                                                    black_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                } else {
                                                    white_orthogonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                }
                                            } else {
                                                if p.white_piece {
                                                    black_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                } else {
                                                    white_diagonal_pin_bitboard |= potential_bitboard | (1 << i) ^ (1 << I + offset);
                                                }
                                            }

                                            break;
                                        } else {
                                            check_number += 1;

                                            if p.white_piece {
                                                black_check_block_bitboard |= potential_bitboard | 1 << i;
                                            } else {
                                                white_check_block_bitboard |= potential_bitboard | 1 << i;
                                            }
                                        }
                                    }
                                }

                                offset += DIR_INDEX_TO_OFFSET[dir];
                            }

                            if !pin_detection {
                                if p.white_piece {
                                    white_attack_bitboard |= potential_bitboard;
                                } else {
                                    black_attack_bitboard |= potential_bitboard;
                                }
                            }
                        }
                    }
                    PieceType::Knight => {
                        for dir in 0..8 {
                            if !self.knight_jump_legality[i as usize][dir] {
                                continue;
                            }

                            if p.white_piece {
                                white_attack_bitboard |= 1 << I + KNIGHT_JUMP_OFFSETS[dir];
                            } else {
                                black_attack_bitboard |= 1 << I + KNIGHT_JUMP_OFFSETS[dir];
                            }

                            if self.squares[(I + KNIGHT_JUMP_OFFSETS[dir]) as usize].is_some() {
                                if square_contains!(self, I + KNIGHT_JUMP_OFFSETS[dir], !p.white_piece, PieceType::King) {
                                    check_number += 1;

                                    if p.white_piece {
                                        black_check_block_bitboard |= (1 << I + KNIGHT_JUMP_OFFSETS[dir]) | (1 << i);
                                    } else {
                                        white_check_block_bitboard |= (1 << I + KNIGHT_JUMP_OFFSETS[dir]) | (1 << i);
                                    }
                                }
                            }
                        }
                    }
                    PieceType::Pawn => {
                        if i < 8 || i > 55 {
                            continue;
                        }

                        let row_in_front: u8 = if p.white_piece { i + 8 } else { i - 8 };

                        if i % 8 != 0 {
                            if p.white_piece {
                                white_attack_bitboard |= 1 << row_in_front - 1;
                            } else {
                                black_attack_bitboard |= 1 << row_in_front - 1;
                            }

                            if square_contains!(self, row_in_front - 1, !p.white_piece, PieceType::King) {
                                if p.white_piece {
                                    black_check_block_bitboard |= (1 << row_in_front - 1) | (1 << i);
                                } else {
                                    white_check_block_bitboard |= (1 << row_in_front - 1) | (1 << i);
                                }
                            }
                        }
                        if i % 8 != 7 {
                            if p.white_piece {
                                white_attack_bitboard |= 1 << row_in_front + 1;
                            } else {
                                black_attack_bitboard |= 1 << row_in_front + 1;
                            }

                            if square_contains!(self, row_in_front + 1, !p.white_piece, PieceType::King) {
                                if p.white_piece {
                                    black_check_block_bitboard |= (1 << row_in_front + 1) | (1 << i);
                                } else {
                                    white_check_block_bitboard |= (1 << row_in_front + 1) | (1 << i);
                                }
                            }
                        }
                    }
                    _ => {
                        eprintln!("Piece on square ({}, {}) set to None", i % 8, i / 8);
                    }
                }
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
}