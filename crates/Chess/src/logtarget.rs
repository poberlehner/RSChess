use std::{fs::File, io::{Read, Write}, path::Path};
use log::LevelFilter;

#[derive(Clone)]
pub struct LogTarget {
    log_file: String,
    pub file_log_level: LevelFilter,
    pub console_log_level: LevelFilter,
}

impl LogTarget {
    pub fn new(log_file: String, file_log_level: LevelFilter, console_log_level: LevelFilter) -> LogTarget {
        if Path::exists(Path::new(&log_file)) {
            let file = File::options().append(true).create(true).open(&log_file);

            match file {
                Ok(mut file) => {
                    let _ = file.write("\n".as_bytes());
                },
                Err(err) => {
                    eprintln!("Failed to write to log file with error: {err}");
                }
            }
        }
        else {
            eprintln!("The log file {log_file} does not exist");
        }

        LogTarget {
            log_file,
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
                let mut file = File::options().append(true).create(true).open(&self.log_file)?;

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