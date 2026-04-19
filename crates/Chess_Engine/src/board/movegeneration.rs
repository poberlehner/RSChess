use crate::board::Board;
use crate::board::r#move::Move;
use crate::board::piece::PieceType;

impl Board {
    #[inline(always)]
    fn add_move(&mut self, origin: (u8, u8), target: (u8, u8), promotion_type: PieceType, white: bool, pinned_orthogonal: bool, pinned_diagonal: bool, check: bool, moves: &mut Vec<Move>) {
        if check {
            if 1 << Self::index_from_coord(target) & self.get_check_block_bitboard(white) == 0 {
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
            if (origin.0.abs_diff(target.0) != 0) ^ (origin.1.abs_diff(target.1) != 0) {
                if 1 << Self::index_from_coord(target) & self.get_orthogonal_pin_bitboard(white) != 0 {
                    moves.push(Move { origin_square: origin, destination_square: target, promotion_type });
                }
            }
        }
        else if pinned_diagonal {
            //Only allow diagonal movement
            if (origin.0.abs_diff(target.0) != 0) & (origin.1.abs_diff(target.1) != 0) {
                if 1 << Self::index_from_coord(target) & self.get_diagonal_pin_bitboard(white) != 0 {
                    moves.push( Move { origin_square: origin, destination_square: target, promotion_type });
                }
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
        
        let mut moves: Vec<Move> = vec![];
        moves.reserve(30);
        let check: bool;

        if white {
            check = 1 << Self::index_from_coord(self.white_king_pos) & self.get_attacked_squares_bitboard(false) != 0;
        }
        else {
            check = 1 << Self::index_from_coord(self.black_king_pos) & self.get_attacked_squares_bitboard(true) != 0;
        }

        for x in 0..8 {
            for y in 0..8 {
                if self.squares[x][y].is_some() {
                    let piece = self.squares[x][y].unwrap();

                    if piece.white_piece != white {
                        continue;
                    }

                    let pinned_orthogonal: bool = 1 << Self::index_from_coord((x as u8, y as u8)) & self.get_orthogonal_pin_bitboard(white) != 0;
                    let pinned_diagonal: bool = 1 << Self::index_from_coord((x as u8, y as u8)) & self.get_diagonal_pin_bitboard(white) != 0;

                    match piece.piece_type {
                        PieceType::King => {
                            for i in -1i8..2 {
                                for j in -1i8..2 {
                                    if i != 0 || j != 0 {
                                        if x as i8 + i < 0 || x as i8 + i > 7 || y as i8 + j < 0 || y as i8 + j > 7 {
                                            continue;
                                        }

                                        if ((!(self.get_attacked_squares_bitboard(
                                            !self.squares[x][y].unwrap().white_piece) |
                                            self.generate_pieces_bitboard(white))) &
                                            (1 << Self::index_from_coord(((x as i8 + i) as u8, (y as i8 + j) as u8)))) != 0
                                        {
                                            moves.push(Move { origin_square: (x as u8, y as u8), destination_square: ((x as i8 + i) as u8, (y as i8 + j) as u8), promotion_type: PieceType::None});
                                        }
                                    }
                                }
                            }
                        }
                        PieceType::Queen => {
                            for direction in [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                                let mut check_offset = (0i8, 0i8);

                                if ((check_offset.0 + direction.0 + x as i8) < 0) || check_offset.0 + direction.0 + x as i8 > 7 {
                                    continue;
                                }
                                if ((check_offset.1 + direction.1 + y as i8) < 0) || check_offset.1 + direction.1 + y as i8 > 7 {
                                    continue;
                                }

                                loop {
                                    check_offset.0 += direction.0;
                                    check_offset.1 += direction.1;

                                    if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].is_none() {
                                        self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                    else {
                                        if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                            self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                        break;
                                    }

                                    if x as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
                                    if x as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
                                    if y as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
                                    if y as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
                                }
                            }
                        }
                        PieceType::Rook => {
                            for direction in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                                let mut check_offset = (0i8, 0i8);

                                if ((check_offset.0 + direction.0 + x as i8) < 0) || check_offset.0 + direction.0 + x as i8 > 7 {
                                    continue;
                                }
                                if ((check_offset.1 + direction.1 + y as i8) < 0) || check_offset.1 + direction.1 + y as i8 > 7 {
                                    continue;
                                }

                                loop {
                                    check_offset.0 += direction.0;
                                    check_offset.1 += direction.1;

                                    if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].is_none() {
                                        self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    } else {
                                        if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                            self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                        break;
                                    }

                                    if x as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
                                    if x as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
                                    if y as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
                                    if y as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
                                }
                            }
                        }
                        PieceType::Bishop => {
                            for direction in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                                let mut check_offset = (0i8, 0i8);

                                if ((check_offset.0 + direction.0 + x as i8) < 0) || check_offset.0 + direction.0 + x as i8 > 7 {
                                    continue;
                                }
                                if ((check_offset.1 + direction.1 + y as i8) < 0) || check_offset.1 + direction.1 + y as i8 > 7 {
                                    continue;
                                }

                                loop {
                                    check_offset.0 += direction.0;
                                    check_offset.1 += direction.1;

                                    if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].is_none() {
                                        self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                    else {
                                        if self.squares[(x as i8 + check_offset.0) as usize][(y as i8 + check_offset.1) as usize].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                            self.add_move((x as u8, y as u8), ((x as i8 + check_offset.0) as u8, (y as i8 + check_offset.1) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                        break;
                                    }

                                    if x as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
                                    if x as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
                                    if y as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
                                    if y as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
                                }
                            }
                        }
                        PieceType::Knight => {
                            for i in [-2i32, -1, 1, 2] {
                                for j in [-2i32, -1, 1, 2] {
                                    if i.abs() != j.abs() {
                                        if x as i32 + i <= 7 && x as i32 + i >= 0 && y as i32 + j <= 7 && y as i32 + j >= 0 {
                                            if self.squares[(x as i32 + i) as usize][(y as i32 + j) as usize].is_some() {
                                                if self.squares[(x as i32 + i) as usize][(y as i32 + j) as usize].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                                    self.add_move((x as u8, y as u8), ((x as i32 + i) as u8, (y as i32 + j) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                                }
                                            }
                                            else {
                                                self.add_move((x as u8, y as u8), ((x as i32 + i) as u8, (y as i32 + j) as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        PieceType::Pawn => {
                            if y == 0 || y == 7 {
                                panic!("A pawn managed to reach the last row");
                            }

                            let row_in_front = y - white as usize + !white as usize;

                            if self.squares[x][row_in_front].is_none() {
                                if row_in_front != 0 && row_in_front != 7 {
                                    self.add_move((x as u8, y as u8), (x as u8, row_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                }

                                if (!piece.white_piece && y == 1) || (piece.white_piece && y == 6) {
                                    let row_2_in_front = row_in_front - white as usize + !white as usize;

                                    if self.squares[x][row_2_in_front].is_none() {
                                        self.add_move((x as u8, y as u8), (x as u8, row_2_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    }
                                }

                                //Promotions
                                if row_in_front == 0 || row_in_front == 7 {
                                    self.add_move((x as u8, y as u8), (x as u8, row_in_front as u8), PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    self.add_move((x as u8, y as u8), (x as u8, row_in_front as u8), PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    self.add_move((x as u8, y as u8), (x as u8, row_in_front as u8), PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                    self.add_move((x as u8, y as u8), (x as u8, row_in_front as u8), PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                }
                            }

                            if x != 7 {
                                if self.squares[x + 1][row_in_front].is_some() {
                                    if self.squares[x + 1][row_in_front].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                        if row_in_front == 0 || row_in_front == 7 {
                                            self.add_move((x as u8, y as u8), ((x + 1) as u8, row_in_front as u8), PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x + 1) as u8, row_in_front as u8), PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x + 1) as u8, row_in_front as u8), PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x + 1) as u8, row_in_front as u8), PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                        else {
                                            self.add_move((x as u8, y as u8), ((x + 1) as u8, row_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                    }
                                }
                                else {
                                    if self.game_state.en_passant_square.is_some() && self.game_state.en_passant_square.unwrap().0 == (x as u8 + 1) && self.game_state.en_passant_square.unwrap().1 == row_in_front as u8 {
                                        if 1 << Self::index_from_coord((x as u8, y as u8)) & self.get_en_passant_pin_bitboard(white) == 0 {
                                            self.add_move((x as u8, y as u8), (x as u8 + 1, row_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                    }
                                }
                            }
                            if x != 0 {
                                if self.squares[x - 1][row_in_front].is_some() {
                                    if self.squares[x - 1][row_in_front].unwrap().white_piece != self.squares[x][y].unwrap().white_piece {
                                        if row_in_front == 0 || row_in_front == 7 {
                                            self.add_move((x as u8, y as u8), ((x - 1) as u8, row_in_front as u8), PieceType::Queen, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x - 1) as u8, row_in_front as u8), PieceType::Rook, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x - 1) as u8, row_in_front as u8), PieceType::Knight, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                            self.add_move((x as u8, y as u8), ((x - 1) as u8, row_in_front as u8), PieceType::Bishop, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                        else {
                                            self.add_move((x as u8, y as u8), ((x - 1) as u8, row_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                    }
                                }
                                else {
                                    if self.game_state.en_passant_square.is_some() && self.game_state.en_passant_square.unwrap().0 == (x as u8 - 1) && self.game_state.en_passant_square.unwrap().1 == row_in_front as u8 {
                                        if 1 << Self::index_from_coord((x as u8, y as u8)) & self.get_en_passant_pin_bitboard(white) == 0 {
                                            self.add_move((x as u8, y as u8), (x as u8 - 1, row_in_front as u8), PieceType::None, white, pinned_orthogonal, pinned_diagonal, check, &mut moves);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            eprintln!("Piece on square ({}, {}) set to None", x,y);
                        }
                    }
                }
            }
        }

        if !self.check() {
            if self.game_state.castling_rights[0] && white {
                if self.squares[6][7].is_none() && self.squares[5][7].is_none() {
                    if ((1 << Self::index_from_coord((6, 7))) | (1 << Self::index_from_coord((5, 7)))) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: (4, 7),
                            destination_square: (6, 7),
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[1] && white {
                if self.squares[1][7].is_none() && self.squares[2][7].is_none() && self.squares[3][7].is_none() {
                    if ((1 << Self::index_from_coord((1, 7))) | (1 << Self::index_from_coord((2, 7))) | (1 << Self::index_from_coord((3, 7)))) & self.get_attacked_squares_bitboard(!white) == 0 {
                        moves.push(Move {
                            origin_square: (4, 7),
                            destination_square: (2, 7),
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[2] && !white {
                if self.squares[6][0].is_none() && self.squares[5][0].is_none() {
                    if ((1 << Self::index_from_coord((6, 0))) | (1 << Self::index_from_coord((5, 0)))) & self.get_attacked_squares_bitboard(white) == 0 {
                        moves.push(Move {
                            origin_square: (4, 0),
                            destination_square: (6, 0),
                            promotion_type: PieceType::None,
                        });
                    }
                }
            }
            if self.game_state.castling_rights[3] && !white {
                if self.squares[1][0].is_none() && self.squares[2][0].is_none() && self.squares[3][0].is_none() {
                    if ((1 << Self::index_from_coord((1, 0))) | (1 << Self::index_from_coord((2, 0))) | (1 << Self::index_from_coord((3, 0)))) & self.get_attacked_squares_bitboard(white) == 0 {
                        moves.push(Move {
                            origin_square: (4, 0),
                            destination_square: (2, 0),
                            promotion_type: PieceType::None,
                        })
                    }
                }
            }
        }

        self.legal_moves = Some(moves);

        &self.legal_moves
    }

    #[allow(unreachable_code)]
    pub fn get_legal_moves_for_square(&mut self, square: (u8, u8)) -> Vec<Move> {
        //Rewrite by simply filtering all moves
        let mut moves: Vec<Move> = vec![];

        for m in self.get_legal_moves().as_ref().unwrap().iter() {
            if m.origin_square == square {
                moves.push(m.clone());
            }
        }

        moves


        // let mut moves: Vec<Move> = vec![];
        //
        // if self.squares[square.0 as usize][square.1 as usize].is_none() {
        //     return moves;
        // }
        //
        // let white: bool = self.squares[square.0 as usize][square.1 as usize].unwrap().white_piece;
        //
        // if white != self.game_state.white_to_move {
        //     return moves;
        // }
        //
        // match self.squares[square.0 as usize][square.1 as usize].unwrap().piece_type {
        //     PieceType::King => {
        //         for i in -1i8..2 {
        //             for j in -1i8..2 {
        //                 if i != 0 || j != 0 {
        //                     if square.0 as i8 + i < 0 || square.0 as i8 + i > 7 || square.1 as i8 + j < 0 || square.1 as i8 + j > 7 {
        //                         continue;
        //                     }
        //
        //                     if ((!(self.generate_attacked_squares_bitboard(
        //                         !white) |
        //                         self.generate_pieces_bitboard(white))) &
        //                         (1 << Self::index_from_coord(((square.0 as i8 + i) as u8, (square.1 as i8 + j) as u8)))) != 0
        //                     {
        //                         moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + i) as u8, (square.1 as i8 + j) as u8), promotion_type: PieceType::None});
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     PieceType::Queen => {
        //         for direction in [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
        //             let mut check_offset = (0i8, 0i8);
        //
        //             if ((check_offset.0 + direction.0 + square.0 as i8) < 0) || check_offset.0 + direction.0 + square.0 as i8 > 7 {
        //                 continue;
        //             }
        //             if ((check_offset.1 + direction.1 + square.1 as i8) < 0) || check_offset.1 + direction.1 + square.1 as i8 > 7 {
        //                 continue;
        //             }
        //
        //             loop {
        //                 check_offset.0 += direction.0;
        //                 check_offset.1 += direction.1;
        //
        //                 if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].is_none() {
        //                     moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                 }
        //                 else {
        //                     if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].unwrap().white_piece != white {
        //                         moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                     }
        //                     break;
        //                 }
        //
        //                 if square.0 as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                 if square.0 as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
        //             }
        //         }
        //     }
        //     PieceType::Rook => {
        //         for direction in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
        //             let mut check_offset = (0i8, 0i8);
        //
        //             if ((check_offset.0 + direction.0 + square.0 as i8) < 0) || check_offset.0 + direction.0 + square.0 as i8 > 7 {
        //                 continue;
        //             }
        //             if ((check_offset.1 + direction.1 + square.1 as i8) < 0) || check_offset.1 + direction.1 + square.1 as i8 > 7 {
        //                 continue;
        //             }
        //
        //             loop {
        //                 check_offset.0 += direction.0;
        //                 check_offset.1 += direction.1;
        //
        //                 if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].is_none() {
        //                     moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                 }
        //                 else {
        //                     if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].unwrap().white_piece != white {
        //                         moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                     }
        //                     break;
        //                 }
        //
        //                 if square.0 as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                 if square.0 as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
        //             }
        //         }
        //     }
        //     PieceType::Bishop => {
        //         for direction in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
        //             let mut check_offset = (0i8, 0i8);
        //
        //             if ((check_offset.0 + direction.0 + square.0 as i8) < 0) || check_offset.0 + direction.0 + square.0 as i8 > 7 {
        //                 continue;
        //             }
        //             if ((check_offset.1 + direction.1 + square.1 as i8) < 0) || check_offset.1 + direction.1 + square.1 as i8 > 7 {
        //                 continue;
        //             }
        //
        //             loop {
        //                 check_offset.0 += direction.0;
        //                 check_offset.1 += direction.1;
        //
        //                 if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].is_none() {
        //                     moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                 }
        //                 else {
        //                     if self.squares[(square.0 as i8 + check_offset.0) as usize][(square.1 as i8 + check_offset.1) as usize].unwrap().white_piece != white {
        //                         moves.push(Move { origin_square: square, destination_square: ((square.0 as i8 + check_offset.0) as u8, (square.1 as i8 + check_offset.1) as u8), promotion_type: PieceType::None});
        //                     }
        //                     break;
        //                 }
        //
        //                 if square.0 as i8 + check_offset.0 == 7 && direction.0 == 1 { break; }
        //                 if square.0 as i8 + check_offset.0 == 0 && direction.0 == -1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 7 && direction.1 == 1 { break; }
        //                 if square.1 as i8 + check_offset.1 == 0 && direction.1 == -1 { break; }
        //             }
        //         }
        //     }
        //     PieceType::Knight => {
        //         for i in [-2i32, -1, 1, 2] {
        //             for j in [-2i32, -1, 1, 2] {
        //                 if i.abs() != j.abs() {
        //                     if square.0 as i32 + i <= 7 && square.0 as i32 + i >= 0 && square.1 as i32 + j <= 7 && square.1 as i32 + j >= 0 {
        //                         if self.squares[(square.0 as i32 + i) as usize][(square.1 as i32 + j) as usize].is_some() {
        //                             if self.squares[(square.0 as i32 + i) as usize][(square.1 as i32 + j) as usize].unwrap().white_piece != white {
        //                                 moves.push(Move { origin_square: square, destination_square: ((square.0 as i32 + i) as u8, (square.1 as i32 + j) as u8), promotion_type: PieceType::None});
        //                             }
        //                         }
        //                         else {
        //                             moves.push(Move { origin_square: square, destination_square: ((square.0 as i32 + i) as u8, (square.1 as i32 + j) as u8), promotion_type: PieceType::None});
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     PieceType::Pawn => {
        //         if square.1 == 0 || square.1 == 7 {
        //             return moves;
        //         }
        //         let row_in_front = square.1 as usize - white as usize + !white as usize;
        //
        //         if self.squares[square.0 as usize][row_in_front].is_none() {
        //             moves.push(Move { origin_square: square, destination_square: (square.0, row_in_front as u8), promotion_type: PieceType::None});
        //
        //             if (!white && square.1 == 1) || (white && square.1 == 6) {
        //                 let row_2_in_front = row_in_front - white as usize + !white as usize;
        //
        //                 if self.squares[square.0 as usize][row_2_in_front].is_none() {
        //                     moves.push(Move { origin_square: square, destination_square: (square.0, row_2_in_front as u8), promotion_type: PieceType::None});
        //                 }
        //             }
        //         }
        //
        //         if square.0 != 7 {
        //             if self.squares[square.0 as usize + 1][row_in_front].is_some() {
        //                 if self.squares[square.0 as usize + 1][row_in_front].unwrap().white_piece != white {
        //                     moves.push(Move { origin_square: square, destination_square: (square.0 + 1, row_in_front as u8), promotion_type: PieceType::None});
        //                 }
        //             }
        //
        //             if self.game_state.en_passant_square.is_some() && self.game_state.en_passant_square.unwrap().0 == (square.0 + 1) && self.game_state.en_passant_square.unwrap().1 == row_in_front as u8 {
        //                 moves.push(Move { origin_square: square, destination_square: (square.0 + 1, row_in_front as u8), promotion_type: PieceType::None});
        //             }
        //         }
        //         if square.0 != 0 {
        //             if self.squares[square.0 as usize - 1][row_in_front].is_some() {
        //                 if self.squares[square.0 as usize - 1][row_in_front].unwrap().white_piece != white {
        //                     moves.push(Move { origin_square: square, destination_square: (square.0 - 1, row_in_front as u8), promotion_type: PieceType::None});
        //                 }
        //             }
        //
        //             if self.game_state.en_passant_square.is_some() && self.game_state.en_passant_square.unwrap().0 == (square.0 - 1) && self.game_state.en_passant_square.unwrap().1 == row_in_front as u8 {
        //                 moves.push(Move { origin_square: square, destination_square: (square.0 - 1, row_in_front as u8), promotion_type: PieceType::None});
        //             }
        //         }
        //     }
        //     _ => {
        //         eprintln!("Piece on square ({}, {}) set to None", square.0, square.1);
        //     }
        // }
        //
        // if square.0 == 4 && square.1 == 7 {
        //     if self.game_state.castling_rights[0] {
        //         if self.squares[6][7].is_none() && self.squares[5][7].is_none() {
        //             if (self.generate_attacked_squares_bitboard(!white) & (1 << Self::index_from_coord((6, 7)) | 1 << Self::index_from_coord((5, 7)))) == 0 {
        //                 moves.push(Move {
        //                     origin_square: (4, 7),
        //                     destination_square: (6, 7),
        //                     promotion_type: PieceType::None,
        //                 });
        //             }
        //         }
        //     }
        //     if self.game_state.castling_rights[1] {
        //         if self.squares[1][7].is_none() && self.squares[2][7].is_none() && self.squares[3][7].is_none() {
        //             if (self.generate_attacked_squares_bitboard(!white) & (1 << Self::index_from_coord((2, 7)) | 1 << Self::index_from_coord((3, 7)))) == 0 {
        //                 moves.push(Move {
        //                     origin_square: (4, 7),
        //                     destination_square: (2, 7),
        //                     promotion_type: PieceType::None,
        //                 });
        //             }
        //         }
        //     }
        // }
        // if square.0 == 4 && square.1 == 0 {
        //     if self.game_state.castling_rights[2] {
        //         if self.squares[6][0].is_none() && self.squares[5][0].is_none() {
        //             if (self.generate_attacked_squares_bitboard(!white) & (1 << Self::index_from_coord((6, 0)) | 1 << Self::index_from_coord((5, 0)))) == 0 {
        //                 moves.push(Move {
        //                     origin_square: (4, 0),
        //                     destination_square: (6, 0),
        //                     promotion_type: PieceType::None,
        //                 });
        //             }
        //         }
        //     }
        //     if self.game_state.castling_rights[3] {
        //         if self.squares[1][0].is_none() && self.squares[2][0].is_none() && self.squares[3][0].is_none() {
        //             if (self.generate_attacked_squares_bitboard(!white) & (1 << Self::index_from_coord((2, 0)) | 1 << Self::index_from_coord((3, 0)))) == 0 {
        //                 moves.push(Move {
        //                     origin_square: (4, 0),
        //                     destination_square: (2, 0),
        //                     promotion_type: PieceType::None,
        //                 })
        //             }
        //         }
        //     }
        // }
        //
        // let mut true_moves: Vec<Move> = Vec::new();
        // if self.generate_king_pin_bitboard(white) & 1 << Self::index_from_coord(square) != 0 {
        //     for m in moves.iter() {
        //         if self.generate_king_pin_bitboard(white) & 1 << Self::index_from_coord(m.destination_square) != 0 {
        //             true_moves.push(m.clone());
        //         }
        //     }
        //
        //     return true_moves;
        // }
        //
        // if self.check() {
        //     let mut true_moves_2: Vec<Move> = Vec::new();
        //
        //     for m in moves.iter() {
        //         if (self.generate_check_block_bitboard(white) & 1 << Self::index_from_coord(m.destination_square)) != 0 {
        //             true_moves_2.push(m.clone());
        //         }
        //         if self.generate_check_block_bitboard(white) & 1 << Self::index_from_coord(m.origin_square) != 0 {
        //             true_moves_2.push(m.clone());
        //         }
        //     }
        //
        //     return true_moves_2;
        // }
        //
        // moves
    }
}