use crate::board::gamestate::GameResult;
use crate::board::r#move::Move;
use crate::board::Board;
use crate::board::piece::PieceType;
use crate::board::utilities::TimeControl;
use crate::renderer::Renderer;
use raylib::prelude::*;

impl Renderer {
    pub fn draw_board(&mut self, draw_handle: &mut RaylibDrawHandle) {
        self.textures.as_ref().expect("Textures are not properly loaded");
        let offset = draw_handle.get_render_width() - self.square_size as i32 * 8;

        for y in 0..8 {
            for x in 0..8 {
                if (x + 8 - y) % 2 == 0 {
                    draw_handle.draw_rectangle((x * self.square_size) as i32 + offset, (y * self.square_size) as i32,
                        self.square_size as i32, self.square_size as i32, self.white_square_color);
                }
                else {
                    draw_handle.draw_rectangle((x * self.square_size) as i32 + offset, (y * self.square_size) as i32,
                        self.square_size as i32, self.square_size as i32, self.black_square_color);
                }
            }
        }
    }

    pub fn draw_pieces(&mut self, board: &Board, draw_handle: &mut RaylibDrawHandle) {
        for i in 0..64 {
            if board.squares[i].is_some() {
                if self.selected_square.is_some() && self.active_selection {
                    if self.selected_square.unwrap() == i as u8 {
                        continue;
                    }
                }

                let iswhite: bool = board.squares[i].unwrap().white_piece;

                draw_handle.draw_texture_ex(&self.textures.as_ref().unwrap()
                    [(board.squares[i].unwrap().piece_type as usize - 1) + 6 * !iswhite as usize],
                    self.square_index_to_pos(&draw_handle, i as i32),
                    0.0, self.square_size as f32 / self.textures.as_ref().unwrap()
                    [(board.squares[i].unwrap().piece_type as usize - 1) +
                    6 * !iswhite as usize].height as f32, Color::WHITE);
            }
        }

        if self.selected_square.is_some() && self.active_selection {
            if board.squares[self.selected_square.unwrap() as usize].is_some() {
                let texture: &Texture2D =
                    &self.textures.as_ref().unwrap()
                        [(board.squares[self.selected_square.unwrap() as usize].unwrap().piece_type as usize - 1) +
                        6 * !board.squares[self.selected_square.unwrap() as usize].unwrap().white_piece as usize];
                let pos: Vector2 = draw_handle.get_mouse_position() - Vector2::new(self.square_size as f32 / 2.0, self.square_size as f32 / 2.0);
                draw_handle.draw_texture_ex(texture, pos, 0.0, self.square_size as f32 / texture.height as f32, Color::WHITE);
            }
        }

        match board.get_time_control() {
            TimeControl::Limited(_, _) => {
                let time_width: i32 = 200;
                draw_handle.draw_text(format!("{}", board.get_time_left(false).as_millis() as f32 / 1000f32).as_str(),
                   (draw_handle.get_screen_width() - self.square_size as i32 * 8 - time_width) as i32, 75, 50, Color::WHITE);

                draw_handle.draw_text(format!("{}", board.get_time_left(true).as_millis() as f32 / 1000f32).as_str(),
                   (draw_handle.get_screen_width() - self.square_size as i32 * 8 - time_width) as i32,
                  (self.square_size * 8 - 75) as i32, 50, Color::WHITE);
            }
            TimeControl::Unlimited => { }
        }

        draw_handle.draw_text(format!("X index: {}, Y index: {}, Index: {}", self.mouse_pos_to_square_index(draw_handle) % 8,
            self.mouse_pos_to_square_index(draw_handle) / 8, self.mouse_pos_to_square_index(draw_handle)).as_str(),100 , 150, 50, Color::WHITE);
    }

    //Draws the moves of the selected piece when moves is None
    pub fn draw_moves(&mut self, board: &mut Board, color: Color, moves: Option<Vec<Move>>, draw_handle: &mut RaylibDrawHandle) {
        if self.promotion_popup.is_some() {
            self.promotion_popup.as_ref().unwrap().draw(draw_handle, &self.textures.as_ref().unwrap());
        }

        if moves.is_some() {
            for m in moves.unwrap() {
                draw_handle.draw_circle((self.square_index_to_pos(&draw_handle, m.destination_square as i32).x + (self.square_size / 2) as f32) as i32,
                    (self.square_index_to_pos(&draw_handle, m.destination_square as i32).y + (self.square_size as i32 / 2) as f32) as i32, self.square_size as f32 / 8.0, color);
            }
        }
        else {
            if self.selected_square.is_some() {
                let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());

                for m in moves {
                    if m.promotion_type == PieceType::None || m.promotion_type == PieceType::Pawn {
                        draw_handle.draw_circle((self.square_index_to_pos(&draw_handle, m.destination_square as i32).x + (self.square_size / 2) as f32) as i32,
                            (self.square_index_to_pos(&draw_handle, m.destination_square as i32).y + (self.square_size / 2) as f32) as i32, self.square_size as f32 / 8.0, color);
                    }
                }
            }
            else {
                return;
            }
        }
    }

    //The shortcuts are handled by Renderer::update_gui()
    pub fn draw_keyboard_shortcut_tips(&mut self, board: &mut Board, draw_handle: &mut RaylibDrawHandle) {
        match board.get_game_result() {
            GameResult::NotStarted => {
                draw_handle.draw_text("Press S to start the Game", 20, 50, 20, Color::WHITE);
            }
            _ => {
                draw_handle.draw_text(format!("Checkmate: {}", board.checkmate() as i32).as_str(),
                    20, 80, 20, Color::WHITE);
                draw_handle.draw_text(format!("Number of Moves: {}", board.get_legal_moves().as_ref().unwrap().len()).as_str(),
                    20, 110, 20, Color::WHITE);
                draw_handle.draw_text("Press R to flip the board", 20, 50, 20, Color::WHITE);
            }
        }
    }

    pub fn draw_bitboard(&mut self, color: Color, draw_handle: &mut RaylibDrawHandle, bitboard: i64) {
        for i in 0..64 {
            if (bitboard & (1i64 << i)) != 0 {
                draw_handle.draw_rectangle(self.square_index_to_pos(&draw_handle, i).x as i32, self.square_index_to_pos(&draw_handle, i).y as i32,
                    self.square_size as i32, self.square_size as i32, color);
            }
        }
    }
}