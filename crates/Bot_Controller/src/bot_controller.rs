use chess_engine::board::{
    STARTFEN,
    gamestate::GameResult,
    r#move::Move,
    utilities::TimeControl
};
use chrono::prelude::*;
use log::*;
use raylib::core::RaylibHandle;
use raylib::ffi::{GetRandomValue, GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE};
use raylib::prelude::*;
use raylib::prelude::TraceLogLevel;
use std::{
    env,
    fs::*,
    io::Write,
    panic,
    panic::PanicHookInfo,
    path::PathBuf,
    thread::{JoinHandle, spawn},
    time::Instant
};
use chess_engine::board::Board;
use chess_engine::renderer::Renderer;
use chess_bot::bot::Bot;
use chess_engine::bot::ChessBot;

fn custom_panic_hook(panic_info: &PanicHookInfo) {
    if let Ok(mut log_file) = OpenOptions::new().append(true).create(true).open("assets/Chess.log") {

        log_file.write_all("[PANIC] ".as_ref()).expect("Failed to write to log file");

         let time = Local::now();
         let time_str = time.to_string();

         log_file.write_all(["[".as_bytes(), time_str.as_bytes(), "] : ".as_bytes(),
             panic_info.payload_as_str().expect("Failed to get panic info as string").as_bytes(), "\n".as_bytes()]
             .concat().as_ref()).expect("Failed to write to log file");
     }
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

        Self::initialize_working_directory();

        rl.set_target_fps(120);
        info!("Window creation took {}ms", start_time.elapsed().as_millis());

        let board = Board::new();

        let renderer = Renderer::new(&mut rl, &thread, "assets/Pieces.png".parse().unwrap(),
            130,Color { r: 171, g: 121, b: 101, a: 255 },
            Color { r: 238, g: 216, b: 192, a: 255});

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

    fn initialize_working_directory() {
        let current_directory = env::current_dir();

        if let Ok(mut working_dir) = current_directory {
            while !(exists(working_dir.join(PathBuf::from("assets"))).is_ok() && exists(working_dir.join(PathBuf::from("assets"))).unwrap()) {
                working_dir.pop();
            }

            env::set_current_dir(working_dir).expect("Failed to set current working directory");
        }
        else {
            error!("Failed to get current working directory");
            error!("This might cause assets, logs or other files to not be loaded or saved correctly");
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

            if self.bot_thread.is_some() && self.bot_thread.as_ref().unwrap().is_finished() {
                let m = self.bot_thread.take().unwrap().join();

                if let Err(err) = m {
                    panic::resume_unwind(err);
                }

                self.board.make_move(m.unwrap(), false);

                if self.board.get_game_result_simplified() != -1 {
                    let competitors = if self.bot_color_white { ("Human", "RSChess Bot") } else { ("RSChess Bot", "Human") };
                    self.board.save_game(String::from("assets/Games.pgn"),
                        Some("RSChess Human vs. Bot"), Some(competitors), None);
                }

                self.bot_thread = None;
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

            if self.board.get_game_result_simplified() != -1 {
                self.board.save_game(String::from("assets/Games.pgn"),
                    Some("RSChess Bot vs. Bot"), Some(("RSChess Bot", "RSChess Bot")), None);
                self.board.load_fen(STARTFEN);
                self.board.start_game(TimeControl::Unlimited);
            }

            if self.bot_thread.is_some() && self.bot_thread.as_ref().unwrap().is_finished() {
                let m = self.bot_thread.take().unwrap().join();

                if let Err(err) = m {
                    panic::resume_unwind(err);
                }

                self.board.make_move(m.unwrap(), false);

                self.bot_thread = None;
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

        self.renderer.unload_textures();
    }
}

impl Default for BotController {
    fn default() -> Self {
        Self::new()
    }
}