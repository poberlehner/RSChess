use chess_engine::board::gamestate::GameResult;
use chess_engine::board::r#move::Move;
use chess_engine::board::utilities::TimeControl;
use chrono::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
use raylib::core::RaylibHandle;
use raylib::ffi::GetRandomValue;
use raylib::ffi::GuiControl::DEFAULT;
use raylib::ffi::GuiDefaultProperty::TEXT_SIZE;
use raylib::prelude::*;
use std::env;
use std::fs::*;
use std::io::Write;
use std::panic;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::thread::spawn;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use chess_engine::board::Board;
use chess_engine::renderer::Renderer;
use chess_bot::bot::Bot;
use chess_engine::bot::ChessBot;

fn custom_trace_log_callback(log_level: TraceLogLevel, string: &str) {
    if let Ok(mut log_file) = OpenOptions::new().append(true).create(true).write(true).open("assets/Chess.log") {

        match log_level {
            TraceLogLevel::LOG_ALL => {
                log_file.write("[FULL] : ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_TRACE => {
                log_file.write("[TRACE] ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_DEBUG => {
                log_file.write("[DEBUG] ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_INFO => {
                log_file.write("[INFO] ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_WARNING => {
                log_file.write("[WARN] ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_ERROR => {
                log_file.write("[ERROR] ".as_ref()).expect("Failed to write to log file");
            }
            TraceLogLevel::LOG_FATAL => {
                log_file.write("[FATAL] ".as_ref()).expect("Failed to write to log file");
            }
            _ => { }
        }

        let time = Local::now();
        let time_str = time.to_string();

        log_file.write(["[".as_bytes(), time_str.as_bytes(), "] : ".as_bytes(), string.as_bytes(), "\n".as_bytes()].concat().as_ref()).expect("Failed to write to log file");
    }
}

fn custom_panic_hook(panic_info: &PanicHookInfo) {
    if let Ok(mut log_file) = OpenOptions::new().append(true).create(true).write(true).open("assets/Chess.log") {

        log_file.write("[PANIC] ".as_ref()).expect("Failed to write to log file");

         let time = Local::now();
         let time_str = time.to_string();

         log_file.write(["[".as_bytes(), time_str.as_bytes(), "] : ".as_bytes(),
             panic_info.payload_as_str().expect("Failed to get panic info as string").as_bytes(), "\n".as_bytes()]
             .concat().as_ref()).expect("Failed to write to log file");
     }
}

pub struct BotController {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,

    ///Only for the bot as the bot_thread keeps it locked for it's entire runtime
    bot_rng: Arc<Mutex<StdRng>>,

    board: Board,
    renderer: Renderer,

    bot_thread: Option<JoinHandle<Move>>,

    bot_color_white: bool,

    bot: Bot
}

impl BotController {
    pub fn new() -> BotController {
        let mut bot_controller = Self::initialize_context();

        bot_controller.initialize_chess();

        bot_controller
    }

    fn initialize_context() -> BotController {
        let start_time = Instant::now();
        let (mut rl, thread) = init()
            .size(1920, 1080)
            .title("Chess")
            .resizable()
            .msaa_4x()
            .build();

        rl.gui_set_style(DEFAULT, TEXT_SIZE, 20);

        set_trace_log_callback(custom_trace_log_callback).expect("Unable to set custom logging functions");

        panic::set_hook(Box::new(custom_panic_hook));

        Self::initialize_working_directory();

        rl.set_target_fps(120);
        trace_log(TraceLogLevel::LOG_INFO, format!("Controller init: Window init time: {:?}", start_time.elapsed()).as_str());

        let start_time = Instant::now();
        let board = Board::new(Some(&mut rl));
        trace_log(TraceLogLevel::LOG_INFO, format!("Controller init: Board init time: {:?}", start_time.elapsed()).as_str());

        let renderer = Renderer::new(&mut rl, &thread, "assets/Pieces.png".parse().unwrap(),
            130,Color { r: 171, g: 121, b: 101, a: 255 },
            Color { r: 238, g: 216, b: 192, a: 255});

        BotController {
            raylib_handle: rl,
            raylib_thread: thread,

            bot_rng: Arc::new(Mutex::new(StdRng::seed_from_u64(15))),

            board,
            renderer,

            bot_thread: None,

            bot_color_white: true,

            bot: Bot::new()
        }
    }

    fn initialize_working_directory() {
        //Handling the working directory
        let current_directory = env::current_dir();
        if current_directory.is_err() {
            trace_log(TraceLogLevel::LOG_FATAL, "Failed to get current working directory");
        }
        let mut current_directory = current_directory.unwrap();

        while !(exists(current_directory.join(PathBuf::from("assets"))).is_ok() && exists(current_directory.join(PathBuf::from("assets"))).unwrap()) {
            trace_log(TraceLogLevel::LOG_ERROR, format!("Current directory: {:?}", current_directory).as_str());

             current_directory.pop();
        }

        env::set_current_dir(current_directory).expect("Failed to set current working directory");
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
                let mut bot_copy = self.bot.clone();
                let rng_copy = self.bot_rng.clone();
                self.bot_thread = Some(spawn(move || -> Move {
                    //Constantly locking self.rng is not ideal because it will lead to lagging if it is used outside while this thread is running
                    bot_copy.get_best_move(&mut bot_board, &mut rng_copy.lock().unwrap())
                }));
            }

            if self.bot_thread.is_some() {
                if self.bot_thread.as_ref().unwrap().is_finished() {
                    let m = self.bot_thread.take().unwrap().join();

                    if m.is_err() {
                        panic::resume_unwind(m.unwrap_err());
                    }

                    self.board.make_move(m.unwrap(), false);

                    self.bot_thread = None;
                }
            }

            self.draw(allow_moving);
        }

        self.renderer.unload_textures();
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
                let mut bot_copy = self.bot.clone();
                let rng_copy = self.bot_rng.clone();
                self.bot_thread = Some(spawn(move || -> Move {
                    //Constantly locking self.rng is not ideal because it will lead to lagging if it is used outside while this thread is running
                    bot_copy.get_best_move(&mut bot_board, &mut rng_copy.lock().unwrap())
                }));
            }

            if self.bot_thread.is_some() {
                if self.bot_thread.as_ref().unwrap().is_finished() {
                    let m = self.bot_thread.take().unwrap().join();

                    if m.is_err() {
                        panic::resume_unwind(m.unwrap_err());
                    }

                    self.board.make_move(m.unwrap(), false);

                    self.bot_thread = None;
                }
            }

            self.draw(false);
        }

        self.renderer.unload_textures();
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

        self.renderer.unload_textures();
    }

    fn draw(&mut self, allow_moving: bool) {
        let mut draw_handle = self.raylib_handle.begin_drawing(&self.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        self.renderer.draw_board(&mut draw_handle);

        self.renderer.draw_pieces(&mut self.board, &mut draw_handle);
        self.renderer.draw_moves(&mut self.board, Color { r: 128, g: 128, b: 128, a: 155}, None, &mut draw_handle);

        self.renderer.draw_keyboard_shortcut_tips(&mut self.board, &mut draw_handle);

        self.renderer.update_gui(&mut self.board, &mut draw_handle, allow_moving);

        draw_handle.draw_fps(20, 20);
    }
}