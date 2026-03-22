use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const LOG_FILE_PATH: &str = "log.txt";

static LOG_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

fn log_file() -> Option<&'static Mutex<File>> {
    LOG_FILE
        .get_or_init(|| {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(LOG_FILE_PATH)
            {
                Ok(file) => Some(Mutex::new(file)),
                Err(err) => {
                    eprintln!("Failed to open {LOG_FILE_PATH}: {err}");
                    None
                }
            }
        })
        .as_ref()
}

fn append_to_log_file(line: &str) {
    let Some(file_mutex) = log_file() else {
        return;
    };

    match file_mutex.lock() {
        Ok(mut file) => {
            if let Err(err) = writeln!(file, "{line}") {
                eprintln!("Failed to write to {LOG_FILE_PATH}: {err}");
            }
        }
        Err(err) => {
            eprintln!("Failed to lock {LOG_FILE_PATH}: {err}");
        }
    }
}

fn write(level: &str, message: &str) {
    let timestamp = Local::now().format(TIMESTAMP_FORMAT);
    let line = format!("[{timestamp}] [{level}] {message}");
    println!("{line}");
    append_to_log_file(&line);
}

pub fn info(message: impl AsRef<str>) {
    write("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write("WARN", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write("ERROR", message.as_ref());
}
