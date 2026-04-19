use chrono::prelude::*;
use raylib::core::RaylibHandle;
use raylib::prelude::*;
use std::env;
use std::fs::*;
use std::io::Write;
use std::panic;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::time::Instant;
use chess_engine::board::Board;
use chess_engine::renderer::Renderer;

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

         log_file.write(["[".as_bytes(), time_str.as_bytes(), "] : ".as_bytes(), panic_info.payload_as_str().expect("Failed to get panic info as string").as_bytes(), "\n".as_bytes()].concat().as_ref()).expect("Failed to write to log file");
     }
}

pub struct Controller {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,

    board: Board,
    renderer: Renderer,
}

impl Controller {
    pub fn new() -> Controller {
        let start_time = Instant::now();
        let (mut rl, thread) = init()
            .size(1920, 1080)
            .title("Chess")
            .resizable()
            .msaa_4x()
            .build();

        set_trace_log_callback(custom_trace_log_callback).expect("Unable to set custom logging functions");

        panic::set_hook(Box::new(custom_panic_hook));

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

        rl.set_target_fps(120);
        trace_log(TraceLogLevel::LOG_INFO, format!("Controller init: Window init time: {:?}", start_time.elapsed()).as_str());

        let start_time = Instant::now();
        let mut board = Board::new(Some(&mut rl));
        trace_log(TraceLogLevel::LOG_INFO, format!("Controller init: Board init time: {:?}", start_time.elapsed()).as_str());

        board.load_fen("rQ2qk1r/1p3ppp/p1p5/8/7b/P7/1PP1B1PP/RNBn1KNR b - - 1 12");

        let renderer = Renderer::new(&mut rl, &thread, "assets/Pieces.png".parse().unwrap(), 130,Color { r: 171, g: 121, b: 101, a: 255 } ,Color { r: 238, g: 216, b: 192, a: 255});

        Controller {
            raylib_handle: rl,
            raylib_thread: thread,

            board,
            renderer,
        }
    }

    pub fn run(&mut self) {
        while !self.raylib_handle.window_should_close() {
            if self.raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || self.raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                if self.raylib_handle.is_key_pressed(KeyboardKey::KEY_F) {
                    self.raylib_handle.toggle_fullscreen();
                }
            }

            self.renderer.update_gui(&mut self.board, &mut self.raylib_handle);

            let mut draw_handle = self.raylib_handle.begin_drawing(&self.raylib_thread);
            draw_handle.clear_background(Color::BLACK);

            self.renderer.draw_board(&mut draw_handle);

            self.renderer.draw_pieces(&mut self.board, &mut draw_handle);
            self.renderer.draw_moves(&mut self.board, Color { r: 128, g: 128, b: 128, a: 155}, None, &mut draw_handle);

            #[cfg(debug_assertions)]
            {
                let bitboard = self.board.get_check_block_bitboard(true);
                let mut color: Color = Color::BLUE;
                color.a = 150;
                self.renderer.draw_bitboard(color, &mut draw_handle, bitboard);
            }

            self.board.get_legal_moves();

            self.renderer.draw_keyboard_shortcut_tips(&mut self.board, &mut draw_handle);

            draw_handle.draw_fps(20, 20);
        }

        self.renderer.unload_textures();
    }
}
