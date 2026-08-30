use std::time::Duration;
use crate::board::gamestate::GameResult;
use crate::board::piece::PieceType;
use crate::board::r#move::Move;
use crate::board::Board;
use crate::board::utilities::TimeControl;
use crate::renderer::promotionpopup::PromotionPopup;
use crate::renderer::Renderer;
use log::*;
use raylib::consts::MouseButton;
use raylib::core::RaylibHandle;
use raylib::drawing::RaylibDrawHandle;
use raylib::ffi::KeyboardKey::{KEY_LEFT, KEY_R, KEY_S, KEY_T};
use raylib::prelude::{Color, Vector2};
use raylib::rgui::RaylibGuiControls;

//Interface functions
impl Renderer {
    /// allow_moving: wether to allow the user to make moves
    pub fn update_gui(&mut self, board: &mut Board, rl: &mut RaylibDrawHandle, allow_moving: bool) {
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.handle_mouse_button_press(allow_moving, board, rl);
        }
        else if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
           self.handle_mouse_button_release(board, rl);
        }

        if rl.is_key_pressed(KEY_S) && board.get_game_result() != GameResult::InProgress {
            board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            board.start_game(TimeControl::Limited(Duration::from_mins(10), Duration::from_millis(1000)));
        }

        if rl.is_key_pressed(KEY_LEFT) {
            board.undo_move(false);
        }

        if rl.is_key_pressed(KEY_R) {
            self.board_flip = !self.board_flip;
        }

        if rl.is_key_pressed(KEY_T) {
            board.test_fen_string_handling();
            board.test_duplicate_moves();
        }

        self.handle_gui_buttons(board, rl);
    }
}

//Functions to split up the UI logic
impl Renderer {
    fn handle_mouse_button_press(&mut self, allow_moving: bool, board: &mut Board, rl: &mut RaylibDrawHandle) {
        let mouse_pos = rl.get_mouse_position();
        let board_offset: f32 = (rl.get_render_width() - self.square_size as i32 * 8) as f32;

        if !allow_moving {
            return;
        }

        if !((mouse_pos.x > board_offset) && (mouse_pos.x < (self.square_size as f32).mul_add(8.0, board_offset) && mouse_pos.y < self.square_size as f32 * 8.0)) {
            self.selected_square = None;
            self.active_selection = false;
            return;
        }

        #[allow(clippy::unwrap_used)]
        if self.promotion_popup.is_some() {
            if self.promotion_move.is_none() {
                error!("Promotion popup exists while promotion_move does not");
                error!("Promotion popup: {:?}", self.promotion_popup.unwrap());
                return
            }
            let promotion_type = self.promotion_popup.unwrap().update(rl);

            if let Some(promotion_type) = promotion_type {
                self.promotion_move =
                    Some( Move {
                        origin_square: self.promotion_move.unwrap().origin_square,
                        destination_square: self.promotion_move.unwrap().destination_square,
                        promotion_type
                    });
                board.make_move(self.promotion_move.unwrap(), false);

                self.promotion_popup = None;
                self.promotion_move = None;
            }
            else {
                warn!("Promotion popup exists while the promotion type does not");
                self.promotion_popup = None;
            }

            return;
        }

        #[allow(clippy::unwrap_used)]
        if self.selected_square.is_some() && !self.active_selection {
            let square: u8 = self.mouse_pos_to_square_index_rl(rl) as u8;
            let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());
            let current_move = Move { origin_square: self.selected_square.unwrap(), destination_square: square, promotion_type: PieceType::None };

            if moves.contains(&current_move) {
                board.make_move(current_move, false);
                self.selected_square = None;
                self.active_selection = false;
            }
            else {
                self.selected_square = Some(
                    self.mouse_pos_to_square_index_rl(rl) as u8
                );

                self.active_selection = true;
            }
        }
        else {
            self.selected_square = Some(
                self.mouse_pos_to_square_index_rl(rl) as u8
            );

            self.active_selection = true;
        }
    }

    #[allow(clippy::unwrap_used)]
    fn handle_mouse_button_release(&mut self, board: &mut Board, rl: &mut RaylibDrawHandle) {
        let square: u8 = self.mouse_pos_to_square_index_rl(rl) as u8;

        if self.selected_square.is_none() {
            return;
        }

        let moves = board.get_legal_moves_for_square(self.selected_square.unwrap());
        let mut current_move = Move { origin_square: self.selected_square.unwrap(), destination_square: square, promotion_type: PieceType::None };

        if moves.contains(&current_move) {
            board.make_move(current_move, false);
            self.selected_square = None;
            self.active_selection = false;
        }
        else {
            //Promotion handling
            current_move.promotion_type = PieceType::Queen;
            if moves.contains(&current_move) {
                let position = self.square_index_to_pos_rl(rl, current_move.origin_square as i32);

                let square = board.squares[current_move.origin_square as usize];

                if let Some(piece) = square {

                    self.promotion_popup = Some(PromotionPopup::new(
                        position,
                        Vector2::new(60f32, 240f32),
                        piece.white_piece,
                        Color::WHITE
                    ));

                    current_move.promotion_type = PieceType::None;
                    self.promotion_move = Some(current_move);
                    self.selected_square = None;
                    self.active_selection = false;
                }
                else {
                    warn!("No piece of the origin of the promoting move");
                }
            }
        }

        self.active_selection = false;
    }

    fn handle_gui_buttons(&self, board: &Board, rl: &mut RaylibDrawHandle) {
        let pressed = rl.gui_label_button(self.pgn_print_button, "Get PGN");

        if pressed {
            println!("{}", board.get_pgn(None, None, None));
        }
    }
}

//Helper functions
impl Renderer {
    pub fn square_index_to_pos(&self, draw_handle: &RaylibDrawHandle, i: i32) -> Vector2 {
        let mut result: Vector2 = Vector2::zero();

        // result.x = ((draw_handle.get_screen_width() - self.square_size as i32 * 8) as i32 + (i % 8) as i32 * self.square_size as i32) as f32;
        // result.y = (self.square_size * 7 - i as u32 / 8 * self.square_size) as f32;

        // if self.board_flip {
        //     result.x = 7f32 - result.x;
        //     result.y = 7f32 - result.y;
        // }

        if self.board_flip {
            result.x = ((draw_handle.get_screen_width() - self.square_size as i32 * 8) + (7 - i % 8) * self.square_size as i32) as f32;
            result.y = (self.square_size * 7 - (7 - i as u32 / 8) * self.square_size) as f32;
        }
        else {
            result.x = ((draw_handle.get_screen_width() - self.square_size as i32 * 8) + (i % 8) * self.square_size as i32) as f32;
            result.y = (self.square_size * 7 - i as u32 / 8 * self.square_size) as f32;
        }

        result
    }

    pub fn square_index_to_pos_rl(&self, rl: &RaylibDrawHandle, i: i32) -> Vector2 {
        let mut result: Vector2 = Vector2::zero();

        if self.board_flip {
            result.x = ((rl.get_screen_width() - self.square_size as i32 * 8) + (7 - i % 8) * self.square_size as i32) as f32;
            result.y = (self.square_size * 7 - (7 - i as u32 / 8) * self.square_size) as f32;
        }
        else {
            result.x = ((rl.get_screen_width() - self.square_size as i32 * 8) + (i % 8) * self.square_size as i32) as f32;
            result.y = (self.square_size * 7 - i as u32 / 8 * self.square_size) as f32;
        }

        result
    }

    ///Not capped at 64
    pub fn mouse_pos_to_square_index(&self, draw_handle: &RaylibDrawHandle) -> i32 {
        let mut x: i32 = ((draw_handle.get_mouse_x() - (draw_handle.get_screen_width() - self.square_size as i32 * 8))
                                                                            as f32 / self.square_size as f32) as i32;
        let mut y: i32 = (self.square_size as i32 * 8 - draw_handle.get_mouse_y()) / self.square_size as i32;

        if self.board_flip {
            x = 7 - x;
            y = 7 - y;
        }

        x + y * 8
    }

    ///Not capped at 64
    pub fn mouse_pos_to_square_index_rl(&self, rl: &RaylibHandle) -> i32 {
        let mut x: i32 = ((rl.get_mouse_x() - (rl.get_screen_width() - self.square_size as i32 * 8)) as f32 / self.square_size as f32) as i32;
        let mut y: i32 = (self.square_size as i32 * 8 - rl.get_mouse_y()) / self.square_size as i32;

        if self.board_flip {
            x = 7 - x;
            y = 7 - y;
        }

        x + y * 8
    }
}