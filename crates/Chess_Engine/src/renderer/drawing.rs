use crate::board::gamestate::GameResult;
use crate::board::r#move::Move;
use crate::board::Board;
use crate::board::utilities::TimeControl;
use crate::renderer::Renderer;
use raylib::prelude::*;

impl Renderer {
    pub fn draw_board(&mut self, draw_handle: &mut RaylibDrawHandle) {
        self.textures.as_ref().expect("Textures are not properly loaded");
        let offset = draw_handle.get_render_width() - self.square_size as i32 * 8;

        for y in 0..8 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
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
        let offset = draw_handle.get_render_width() - self.square_size as i32 * 8;

        for y in 0..8 {
            for x in 0..8 {
                if board.squares[x as usize][y as usize].is_some() {
                    if self.selected_square.is_some() && self.active_selection {
                        if self.selected_square.unwrap().0 == x as u8 && self.selected_square.unwrap().1 == y as u8 {
                            continue;
                        }
                    }

                    let iswhite: bool = board.squares[x as usize][y as usize].unwrap().white_piece;

                    draw_handle.draw_texture_ex(&self.textures.as_ref().unwrap()
                        [(board.squares[x as usize][y as usize].unwrap().piece_type as usize - 1) + 6 * !iswhite as usize],
                        Vector2 { x: offset as f32 + x as f32 * self.square_size as f32, y: y as f32 * self.square_size as f32},
                        0.0, self.square_size as f32 / self.textures.as_ref().unwrap()
                        [(board.squares[x as usize][y as usize].unwrap().piece_type as usize - 1) +
                        6 * !iswhite as usize].height as f32, Color::WHITE);
                }
            }
        }

        if self.selected_square.is_some() && self.active_selection {
            if board.squares[self.selected_square.unwrap().0 as usize][self.selected_square.unwrap().1 as usize].is_some() {
                let texture: &Texture2D =
                    &self.textures.as_ref().unwrap()
                        [(board.squares[self.selected_square.unwrap().0 as usize][self.selected_square.unwrap().1 as usize].unwrap().piece_type as usize - 1) +
                        6 * !board.squares[self.selected_square.unwrap().0 as usize][self.selected_square.unwrap().1 as usize].unwrap().white_piece as usize];
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
    }

    //Draws the moves of the selected piece when moves is None
    pub fn draw_moves(&mut self, board: &mut Board, color: Color, moves: Option<Vec<Move>>, draw_handle: &mut RaylibDrawHandle) {
        let offset = draw_handle.get_render_width() - self.square_size as i32 * 8;

        if self.promotion_popup.is_some() {
            self.promotion_popup.as_ref().unwrap().draw(draw_handle, &self.textures.as_ref().unwrap());
        }

        if moves.is_some() {
            for m in moves.unwrap() {
                draw_handle.draw_circle(offset + self.square_size as i32 * m.destination_square.0 as i32 + (self.square_size / 2) as i32,
                    self.square_size as i32 * m.destination_square.1 as i32 + (self.square_size / 2) as i32, self.square_size as f32 / 8.0, color);
            }
        }
        else {
            if self.selected_square.is_some() {
                let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());

                for m in moves {
                    let pos_x = offset + self.square_size as i32 * m.destination_square.0 as i32 + (self.square_size / 2) as i32;
                    let pos_y = self.square_size as i32 * m.destination_square.1 as i32 + (self.square_size / 2) as i32;
                    draw_handle.draw_circle(pos_x, pos_y, self.square_size as f32 / 8.0, color);
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
            }
        }
    }

    pub fn draw_bitboard(&mut self, color: Color, draw_handle: &mut RaylibDrawHandle, bitboard: i64) {
        let offset = draw_handle.get_render_width() - self.square_size as i32 * 8;

        for y in 0..8 {
            for x in 0..8 {
                if (bitboard & (1i64 << (y * 8 + x))) != 0 {
                    draw_handle.draw_rectangle(offset + self.square_size as i32 * x, self.square_size as i32 * y,
                        self.square_size as i32, self.square_size as i32, color);
                }
            }
        }
    }
}