mod gui;
mod drawing;
mod promotionpopup;

use std::time::Instant;
use log::info;
use raylib::color::Color;
use raylib::math::Rectangle;
use raylib::prelude::*;
use raylib::{RaylibHandle, RaylibThread};
use crate::board::r#move::Move;
use crate::renderer::promotionpopup::PromotionPopup;

pub struct Renderer
{
    //The board is at the top right edge of the screen
    square_size: u32,

    white_square_color: Color,
    black_square_color: Color,

    board_flip: bool,

    pub(super) selected_square: Option<u8>,
    //If the mouse button is still being held
    pub(super) active_selection: bool,

    promotion_popup: Option<PromotionPopup>,
    promotion_move: Option<Move>,

    pgn_print_button: Rectangle,

    //white the black
    //Pawn, Knight, Bishop, Rook, Queen, King
    textures: Option<[Texture2D; 12]>,
}

impl Renderer
{
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread,
               texture_atlas_path: String, square_size: u32, black_square_color: Color,
               white_square_color: Color) -> Self
    {
        let start_time = Instant::now();
        let renderer: Renderer = Renderer {
            textures: Some(Self::load_piece_textures(rl, thread, texture_atlas_path, 334)),
            square_size,

            selected_square: None,
            active_selection: false,

            board_flip: false,

            promotion_popup: None,
            promotion_move: None,

            pgn_print_button: Rectangle::new(400f32, 20f32, 200f32, 100f32),

            black_square_color,
            white_square_color,
        };

        info!("Renderer init took {}ms", start_time.elapsed().as_millis());

        renderer
    }

    /// * `texture_atlas_path` left to right: King, Queen, Bishop, Knight, Pawn   white above black
    /// * `size`: the size of each texture
    ///
    /// returns: [Texture2D; 12]
    ///     the textures for each piece (all white pieces then black) Pawn, Knight, Bishop, Rook, Queen, King
    pub fn load_piece_textures(raylib_handle: &mut RaylibHandle, raylib_thread: &RaylibThread, texture_atlas_path: String, size: u32) -> [Texture2D; 12] {
        let texture_atlas = Image::load_image(&texture_atlas_path).expect("Could not load texture atlas from {texture_atlas_path}");

        let mut buf_image1;
        buf_image1 = texture_atlas.clone();
        buf_image1.crop(Rectangle { x: size as f32 * 5.0, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image2;
        buf_image2 = texture_atlas.clone();
        buf_image2.crop(Rectangle { x: size as f32 * 3.0, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image3;
        buf_image3 = texture_atlas.clone();
        buf_image3.crop(Rectangle { x: size as f32 * 2.0, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image4;
        buf_image4 = texture_atlas.clone();
        buf_image4.crop(Rectangle { x: size as f32 * 4.0, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image5;
        buf_image5 = texture_atlas.clone();
        buf_image5.crop(Rectangle { x: size as f32, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image6;
        buf_image6 = texture_atlas.clone();
        buf_image6.crop(Rectangle { x: 0.0, y: 0.0, width: size as f32, height: size as f32 });

        let mut buf_image7;
        buf_image7 = texture_atlas.clone();
        buf_image7.crop(Rectangle { x: size as f32 * 5.0, y: size as f32, width: size as f32, height: size as f32 });

        let mut buf_image8;
        buf_image8 = texture_atlas.clone();
        buf_image8.crop(Rectangle { x: size as f32 * 3.0, y: size as f32, width: size as f32, height: size as f32 });

        let mut buf_image9;
        buf_image9 = texture_atlas.clone();
        buf_image9.crop(Rectangle { x: size as f32 * 2.0, y: size as f32, width: size as f32, height: size as f32 });

        let mut buf_image10;
        buf_image10 = texture_atlas.clone();
        buf_image10.crop(Rectangle { x: size as f32 * 4.0, y: size as f32, width: size as f32, height: size as f32 });

        let mut buf_image11;
        buf_image11 = texture_atlas.clone();
        buf_image11.crop(Rectangle { x: size as f32, y: size as f32, width: size as f32, height: size as f32 });

        let mut buf_image12;
        buf_image12 = texture_atlas.clone();
        buf_image12.crop(Rectangle { x: 0.0, y: size as f32, width: size as f32, height: size as f32 });

        let textures: [Texture2D; 12] = [raylib_handle.load_texture_from_image(raylib_thread, &buf_image1).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image2).unwrap(),
            raylib_handle.load_texture_from_image(raylib_thread, &buf_image3).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image4).unwrap(),
            raylib_handle.load_texture_from_image(raylib_thread, &buf_image5).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image6).unwrap(),
            raylib_handle.load_texture_from_image(raylib_thread, &buf_image7).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image8).unwrap(),
            raylib_handle.load_texture_from_image(raylib_thread, &buf_image9).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image10).unwrap(),
            raylib_handle.load_texture_from_image(raylib_thread, &buf_image11).unwrap(), raylib_handle.load_texture_from_image(raylib_thread, &buf_image12).unwrap()];

        textures
    }

    pub fn unload_textures(&mut self) {
        self.textures = None;
    }
}