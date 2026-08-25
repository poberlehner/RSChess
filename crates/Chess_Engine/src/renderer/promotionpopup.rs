use raylib::ffi::MouseButton::MOUSE_BUTTON_LEFT;
use raylib::prelude::*;
use crate::board::piece::PieceType;

#[derive(Copy, Clone)]
pub(crate) struct PromotionPopup
{
    pub(crate) position: Vector2,
    //its Width is assumed to be a fourth of the height
    pub(crate) size: Vector2,
    pub(crate) bg_color: Color,
    pub(crate) white: bool
}

impl PromotionPopup
{
    pub fn new(position: Vector2, size: Vector2, white: bool, bg_color: Color) -> Self {
        PromotionPopup {
            position,
            size,
            bg_color,
            white
        }
    }

    pub fn draw(&self, draw_handle: &mut RaylibDrawHandle, textures: &[Texture2D; 12]) {
        let texture_scale: f32 = self.size.x / textures[0].height as f32;

        draw_handle.draw_rectangle_rounded(
            Rectangle {
                x: self.position.x, y: self.position.y, width: self.size.x, height: self.size.y,
            },
            0.4,
            5,
            self.bg_color);

        //These are for the white pieces, apply an offset of 6 for the black ones
        //Queen, Rook, Knight, Bishop
        static INDICES: [i32; 4] = [4, 3, 1, 2];
        for i in 0..4 {
            draw_handle.draw_texture_ex(&textures[(INDICES[i] + (!self.white as i32 * 6)) as usize], Vector2 { x: self.position.x, y: self.position.y + (self.size.y / 4f32) * i as f32 }, 0f32, texture_scale, Color::WHITE);
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) -> Option<PieceType> {
        if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
            let mouse_pos = rl.get_mouse_position();

            if self.position.x < mouse_pos.x && self.position.x + self.size.x > mouse_pos.x {
                let mouse_height_rel = mouse_pos.y - self.position.y;

                if mouse_height_rel < 0f32 {
                    return None;
                }
                if mouse_height_rel > self.size.y {
                    return None;
                }

                let selected = (mouse_height_rel / self.size.x) as i32;

                return match selected {
                    0 => {
                        Some(PieceType::Queen)
                    }
                    1 => {
                        Some(PieceType::Rook)
                    }
                    2 => {
                        Some(PieceType::Knight)
                    }
                    3 => {
                        Some(PieceType::Bishop)
                    }
                    _ => {
                        None
                    }
                }
            }
        }

        None
    }
}