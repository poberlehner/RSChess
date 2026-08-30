use chess_engine::board::{
    STARTFEN,
    gamestate::GameResult,
    r#move::Move,
    utilities::TimeControl
};
use chrono::prelude::*;
use log::{error, info};
use raylib::core::RaylibHandle;
use raylib::ffi::{GetRandomValue, GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE};
use raylib::prelude::*;
use raylib::prelude::TraceLogLevel;
use std::{
    fs::*,
    io::Write,
    panic,
    panic::PanicHookInfo,
    thread::{JoinHandle, spawn},
    time::Instant
};
use chess_engine::board::Board;
use chess_engine::renderer::Renderer;
use chess_bot::bot::Bot;
use chess_engine::bot::ChessBot;

#[allow(clippy::expect_used, clippy::unwrap_used)]
fn custom_panic_hook(panic_info: &PanicHookInfo) {
    if let Ok(mut log_file) = OpenOptions::new().append(true).create(true).open("assets/Chess.log") {

        log_file.write_all("[PANIC] ".as_ref()).expect("Failed to write to log file");

         let time = Local::now();
         let time_str = time.to_string();

         let panic_str = ["[".as_bytes(), time_str.as_bytes(), "] : ".as_bytes(),
             panic_info.payload_as_str().unwrap_or("Invalid panic payload").as_bytes(), "\n".as_bytes()];

         log_file.write_all(panic_str.concat().as_ref()).expect("Failed to write to log file");

         eprintln!("[PANIC] {}", str::from_utf8(panic_str.concat().as_ref()).expect("Invalid utf8 sequence"))
     }

     error!("Failed to open log file");
}

pub struct BotController {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,

    frame_times: Vec<f32>,

    board: Board,
    renderer: Renderer,

    bot_thread: Option<JoinHandle<Move>>,

    bot_color_white: bool,

    bot: Bot
}

#[allow(clippy::collapsible_if)]
impl BotController {
    pub fn new() -> BotController {
        let mut bot_controller = Self::initialize_context();

        bot_controller.initialize_chess();

        bot_controller
    }

    fn initialize_context() -> BotController {
        let start_time = Instant::now();
        let (mut rl, thread) = init()
            .log_level(TraceLogLevel::LOG_WARNING)
            .size(1920, 1080)
            .title("Chess")
            .resizable()
            .msaa_4x()
            .build();

        rl.gui_set_style(DEFAULT, TEXT_SIZE, 20);

        panic::set_hook(Box::new(custom_panic_hook));

        rl.set_target_fps(120);
        info!("Window creation took {}ms", start_time.elapsed().as_millis());

        let board = Board::new();

        let mut renderer = Renderer::new(130,Color { r: 171, g: 121, b: 101, a: 255 },
            Color { r: 238, g: 216, b: 192, a: 255});

        let texture_load_result = renderer.load_piece_textures(&mut rl, &thread, "assets/Pieces.png".parse().unwrap(), 334);

        if let Err(err) = texture_load_result {
            error!("Failed to load piece textures from file \"assets/Pieces.png\" with error {err}");

        }

        BotController {
            raylib_handle: rl,
            raylib_thread: thread,

            frame_times: vec![],

            board,
            renderer,

            bot_thread: None,

            bot_color_white: true,

            bot: Bot::new()
        }
    }

    fn initialize_chess(&mut self) {
        unsafe {
            self.bot_color_white = GetRandomValue(0, 1) != 0;
        }

        self.board.start_game(TimeControl::Unlimited);
    }

    pub fn run_human_vs_bot(&mut self) {
        while !self.raylib_handle.window_should_close() {
            if self.raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || self.raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                if self.raylib_handle.is_key_pressed(KeyboardKey::KEY_F) {
                    self.raylib_handle.toggle_fullscreen();
                }
            }

            //Wether to allow the player to move pieces
            let allow_moving = self.board.get_white_to_move() != self.bot_color_white;

            if !allow_moving && self.board.get_game_result() == GameResult::InProgress && self.bot_thread.is_none() {
                let mut bot_board = self.board.clone();
                let mut bot_copy = self.bot;
                self.bot_thread = Some(spawn(move || -> Move {
                    bot_copy.get_best_move(&mut bot_board)
                }));
            }

            self.handle_bot_thread_finished();

            if self.board.get_game_result_simplified() != -1 {
                let competitors = if self.bot_color_white { ("Human", "RSChess Bot") } else { ("RSChess Bot", "Human") };
                self.board.save_game(String::from("assets/Games.pgn"),
                    Some("RSChess Human vs. Bot"), Some(competitors), None);
            }

            self.draw(allow_moving);
        }
    }

    pub fn run_bot_vs_bot(&mut self) {
        while !self.raylib_handle.window_should_close() {
            if self.raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || self.raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                if self.raylib_handle.is_key_pressed(KeyboardKey::KEY_F) {
                    self.raylib_handle.toggle_fullscreen();
                }
            }

            if self.board.get_game_result() == GameResult::InProgress && self.bot_thread.is_none() {
                let mut bot_board = self.board.clone();
                let mut bot_copy = self.bot;
                self.bot_thread = Some(spawn(move || -> Move {
                    bot_copy.get_best_move(&mut bot_board)
                }));
            }

            self.handle_bot_thread_finished();

            if self.board.get_game_result_simplified() != -1 {
                self.board.save_game(String::from("assets/Games.pgn"),
                    Some("RSChess Bot vs. Bot"), Some(("RSChess Bot", "RSChess Bot")), None);
                self.board.load_fen(STARTFEN);
                self.board.start_game(TimeControl::Unlimited);
            }

            self.draw(false);
        }
    }

    pub fn run_human_vs_human(&mut self) {
        while !self.raylib_handle.window_should_close() {
            if self.raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || self.raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                if self.raylib_handle.is_key_pressed(KeyboardKey::KEY_F) {
                    self.raylib_handle.toggle_fullscreen();
                }
            }

            self.draw(true);
        }
    }

    fn handle_bot_thread_finished(&mut self) {
        if self.bot_thread.is_none() { return }

        #[allow(clippy::unwrap_used)]
        if self.bot_thread.as_ref().unwrap().is_finished() {
            #[allow(clippy::unwrap_used)]
            let m = self.bot_thread.take().unwrap().join();

            match m {
                Err(err) => {
                    panic::resume_unwind(err);
                }
                Ok(m) => {
                    self.board.make_move(m, false);
                }
            }

            self.bot_thread = None;
        }
    }

    fn draw(&mut self, allow_moving: bool) {
        let mut draw_handle = self.raylib_handle.begin_drawing(&self.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        self.renderer.draw_board(&mut draw_handle);

        self.renderer.draw_pieces(&self.board, &mut draw_handle);
        self.renderer.draw_moves(&mut self.board, Color { r: 128, g: 128, b: 128, a: 155}, None, &mut draw_handle);

        self.renderer.draw_keyboard_shortcut_tips(&mut self.board, &mut draw_handle);

        self.renderer.update_gui(&mut self.board, &mut draw_handle, allow_moving);

        draw_handle.draw_fps(20, 20);

        self.frame_times.push(draw_handle.get_frame_time());
    }
}

impl Drop for BotController {
    fn drop(&mut self) {
        self.frame_times.remove(0);
        let sum: f32 = self.frame_times.iter().sum();
        let avg = sum / self.frame_times.len() as f32;
        info!("Average frame times: {avg}s");

        // The textures have to be unloaded before RaylibHandle because it unloads all loaded textures when dropped
        // If the textures themselves are dropped after they will try to unload themselves leading to a double free in the underlying C code
        self.renderer.unload_textures();
    }
}

impl Default for BotController {
    fn default() -> Self {
        Self::new()
    }
}