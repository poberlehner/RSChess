use std::cmp::min;
use std::time::Duration;
use crate::board::gamestate::GameResult;
use crate::board::piece::PieceType;
use crate::board::r#move::Move;
use crate::board::Board;
use crate::board::utilities::TimeControl;
use crate::renderer::promotionpopup::PromotionPopup;
use crate::renderer::Renderer;
use raylib::consts::MouseButton;
use raylib::core::RaylibHandle;
use raylib::ffi::KeyboardKey::{KEY_LEFT, KEY_R, KEY_S, KEY_T};
use raylib::prelude::{Color, TraceLogLevel, Vector2};
use raylib::trace_log;

//Interface functions
impl Renderer {
    pub fn update_gui(&mut self, board: &mut Board, rl: &mut RaylibHandle) {
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.handle_mouse_button_press(board, rl);
        }
        else if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
           self.handle_mouse_button_release(board, rl);
        }

        if rl.is_key_pressed(KEY_S) && board.get_game_result() != GameResult::InProgress {
            board.start_game(true,
                TimeControl::Limited(Duration::from_mins(10), Duration::from_mins(10)));
        }

        if rl.is_key_pressed(KEY_LEFT) {
            board.undo_move();
        }

        if rl.is_key_pressed(KEY_R) {
            board.invalidate_caches();
            board.generate_all_bitboards();
            board.get_legal_moves();
        }

        if rl.is_key_pressed(KEY_T) {
            board.test_fen_string_handling();
        }
    }
}

//Functions to split up the UI logic
impl Renderer {
    fn handle_mouse_button_press(&mut self, board: &mut Board, rl: &mut RaylibHandle) {
        let mouse_pos = rl.get_mouse_position();
        let board_offset: f32 = (rl.get_render_width() - self.square_size as i32 * 8) as f32;

        if !((mouse_pos.x > board_offset) && (mouse_pos.x < (board_offset + self.square_size as f32 * 8.0) && mouse_pos.y < self.square_size as f32 * 8.0)) {
            self.selected_square = None;
            self.active_selection = false;
            return;
        }

        if self.promotion_popup.is_some() {
            let promotion_type = self.promotion_popup.unwrap().update(rl);

            if promotion_type.is_some() {
                self.promotion_move = Some(Move { origin_square: self.promotion_move.unwrap().origin_square, destination_square: self.promotion_move.unwrap().destination_square, promotion_type: promotion_type.unwrap()});
                board.make_move(self.promotion_move.unwrap());

                self.promotion_popup = None;
                self.promotion_move = None;
            }
            else {
                trace_log(TraceLogLevel::LOG_WARNING, "promotion_popup is some while promotion_move is none");
                self.promotion_popup = None;
            }

            return;
        }

        if self.selected_square.is_some() && !self.active_selection {
            let square: (u8, u8) = (((mouse_pos.x - board_offset) as u32 / self.square_size) as u8, (mouse_pos.y as u32 / self.square_size) as u8);
            let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());
            let current_move = Move { origin_square: self.selected_square.unwrap(), destination_square: square, promotion_type: PieceType::None };

            if moves.contains(&current_move) {
                board.make_move(current_move);
                self.selected_square = None;
                self.active_selection = false;
            }
            else {
                self.selected_square = Some(
                    (
                        ((mouse_pos.x - board_offset) as u32 / self.square_size) as u8,
                        (mouse_pos.y as u32 / self.square_size) as u8,
                    )
                );

                self.active_selection = true;
            }
        }
        else {
            self.selected_square = Some(
                (
                    ((mouse_pos.x - board_offset) as u32 / self.square_size) as u8,
                    (mouse_pos.y as u32 / self.square_size) as u8,
                )
            );

            self.active_selection = true;
        }
    }

    fn handle_mouse_button_release(&mut self, board: &mut Board, rl: &mut RaylibHandle) {
        let mouse_pos = rl.get_mouse_position();
        let board_offset: f32 = (rl.get_render_width() - self.square_size as i32 * 8) as f32;

        let square: (u8, u8) = (((mouse_pos.x - board_offset) as u32 / self.square_size) as u8, (mouse_pos.y as u32 / self.square_size) as u8);

        if self.selected_square.is_none() {
            return;
        }

        let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());
        let mut current_move = Move { origin_square: self.selected_square.unwrap(), destination_square: square, promotion_type: PieceType::None };

        if moves.contains(&current_move) {
            board.make_move(current_move);
            self.selected_square = None;
            self.active_selection = false;
        }
        else {
            //Promotion handling
            current_move.promotion_type = PieceType::Queen;
            if moves.contains(&current_move) {
                let size = Vector2::new(60f32, 240f32);
                let position = Vector2 {
                    x: rl.get_screen_width() as f32 - (self.square_size as i32 * (8i32 - (current_move.origin_square.0 + 1) as i32)) as f32 - (self.square_size as f32 / 2f32),
                    y: min((self.square_size as i32 * (current_move.origin_square.1 + 1) as i32) - (self.square_size as f32 / 2f32) as i32, rl.get_screen_height() - size.y as i32) as f32
                };

                self.promotion_popup = Some(PromotionPopup::new(
                    position,
                    Vector2::new(60f32, 240f32),
                    board.squares[current_move.origin_square.0 as usize][current_move.origin_square.1 as usize]
                        .expect("No piece at the origin of a move (Renderer)").white_piece,
                    Color::WHITE
                ));

                current_move.promotion_type = PieceType::None;
                self.promotion_move = Some(current_move);

                self.selected_square = None;
                self.active_selection = false;
            }
        }

        self.active_selection = false;
    }
}