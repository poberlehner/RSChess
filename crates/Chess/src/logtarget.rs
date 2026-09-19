use std::{fs::File, io::{Read, Write}, path::Path};
use log::LevelFilter;

#[derive(Clone)]
pub struct LogTarget {
    log_file_path: String,
    pub file_log_level: LevelFilter,
    pub console_log_level: LevelFilter,
}

impl LogTarget {
    pub fn new(log_file_path: String, file_log_level: LevelFilter, console_log_level: LevelFilter) -> LogTarget {
        let log_dir = Path::new(&log_file_path);

        let mut log_dir_buf = log_dir.to_path_buf();

        log_dir_buf.pop();

        if !log_dir_buf.exists() {
            let r = std::fs::create_dir_all(log_dir_buf);

            if let Err(err) = r {
                eprintln!("Failed to create log directory: \"{err}\"")
            }
        }

        let file = File::options().append(true).create(true).open(&log_file_path);

        match file {
            Ok(mut file) => {
                let _ = file.write("\n".as_bytes());
            }
            Err(err) => {
                eprintln!("Failed to create log file with error: {err}")
            }
        }

        LogTarget {
            log_file_path: log_file_path,
            file_log_level,
            console_log_level,
        }
    }

    fn log_level_from_char(c: char) -> LevelFilter {
        match c {
            'T' => { LevelFilter::Trace }
            'D' => { LevelFilter::Debug }
            'E' => { LevelFilter::Error }
            'O' => { LevelFilter::Off   }
            'I' => { LevelFilter::Info  }
            'W' => { LevelFilter::Warn  }
            _ => {
                println!("Char '{c}' not mapped to log level");
                LevelFilter::Trace
            }
        }
    }
}

impl Write for LogTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut string: String = String::new();
        let result = buf.as_ref().read_to_string(&mut string);

        //Filter for trace log levels
        if buf.len() >= 23 {
            let level = Self::log_level_from_char(buf[22] as char);

            if level <= self.file_log_level {
                let mut file = File::options().append(true).create(true).open(&self.log_file_path)?;

                file.write(buf)?;
            }

            if level <= self.console_log_level {
                print!("{}", string);
            }
        }

        result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}