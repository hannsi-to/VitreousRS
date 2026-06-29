use crate::vitreous_rs::{API_NAME, DEFAULT_PROJECT_NAME};
use crate::{debug_ln, error_ln, info_ln, warning_ln};
use backtrace::Backtrace;
use chrono::Local;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{OnceLock, RwLock};
use std::{fs, panic};

static COUNTER: AtomicI32 = AtomicI32::new(1);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static CREATE_TIME: OnceLock<String> = OnceLock::new();
thread_local! {
    static PANIC_LOGGER: RefCell<Option<Box<dyn Fn(&str)>>> = RefCell::new(None);
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

pub struct LoggerConfig {
    pub project_name: &'static str,
    pub format: &'static str,
    pub time_format: &'static str,
    pub log_file_path: &'static str,
    pub max_file_size: u64,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            project_name: DEFAULT_PROJECT_NAME,
            format: "[{{TIME}}][{{PROJECT_NAME}}][{{LOG_LEVEL}}] {{MESSAGE}}",
            time_format: "%Y/%m/%d %H:%M:%S",
            log_file_path: "logs",
            max_file_size: 1024 * 1024 * 10,
        }
    }
}

pub static LOGGER_MANAGER: RwLock<LoggerConfig> = RwLock::new(LoggerConfig {
    project_name: DEFAULT_PROJECT_NAME,
    format: "[{{TIME}}][{{PROJECT_NAME}}][{{LOG_LEVEL}}] {{MESSAGE}}",
    time_format: "%Y/%m/%d %H:%M:%S",
    log_file_path: "logs",
    max_file_size: 1024 * 1024 * 10,
});

pub struct LoggerManager;

impl LoggerManager {
    pub fn set_project_name(name: &'static str) {
        LOGGER_MANAGER.write().unwrap().project_name = name;
    }

    pub fn set_format(format: &'static str) {
        LOGGER_MANAGER.write().unwrap().format = format;
    }

    pub fn set_time_format(format: &'static str) {
        LOGGER_MANAGER.write().unwrap().time_format = format;
    }

    pub fn set_log_file_path(path: &'static str) {
        LOGGER_MANAGER.write().unwrap().log_file_path = path;
    }

    pub fn set_max_file_size(size: u64) {
        LOGGER_MANAGER.write().unwrap().max_file_size = size;
    }

    pub fn init() {
        CREATE_TIME.get_or_init(|| Local::now().format("%Y-%m-%d").to_string());

        Self::sync_counter();
        Self::setup_panic_hook();
    }

    fn sync_counter() {
        if IS_INITIALIZED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let config = LOGGER_MANAGER.read().unwrap();
        let date = CREATE_TIME.get().map(|s| s.as_str()).unwrap_or("");
        let mut number = 1;

        loop {
            let mut path = PathBuf::from(config.log_file_path);
            path.push(
                format!(
                    "{}_{}_{}_({}).log",
                    config.project_name, API_NAME, date, number
                )
                .replace(" ", "_"),
            );

            if !path.exists() {
                let target = if number > 1 { number - 1 } else { 1 };
                COUNTER.store(target, Ordering::SeqCst);
                break;
            }

            let Ok(metadata) = fs::metadata(&path) else {
                break;
            };

            if metadata.len() < config.max_file_size {
                COUNTER.store(number, Ordering::SeqCst);
                break;
            }

            number += 1;
        }
    }

    pub fn setup_panic_hook() {
        PANIC_LOGGER.with(|logger| {
            *logger.borrow_mut() = Some(Box::new(|message| {
                use std::io::Write;
                let _ = writeln!(std::io::stderr(), "{}", message);
            }));
        });

        panic::set_hook(Box::new(|panic_info| {
            let message = if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else {
                String::from("Box<dyn Any>")
            };

            let location = if let Some(loc) = panic_info.location() {
                format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
            } else {
                String::from("unknown location")
            };

            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");
            let thread_id = std::thread::current().id();

            let backtrace_str = match std::env::var("RUST_BACKTRACE").as_deref() {
                Ok("full") => {
                    let bt = Backtrace::new();
                    format!("\nstack backtrace:\n{:?}", bt)
                }
                Ok("1") => {
                    let bt = Backtrace::new();
                    let frames: Vec<String> = bt
                        .frames()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, frame)| {
                            let symbol = frame.symbols().first()?;
                            let name = symbol
                                .name()
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| String::from("<unknown>"));

                            if name.contains("backtrace")
                                || name.contains("panic")
                                || name.contains("__rust")
                                || name.contains("setup_panic_hook")
                            {
                                return None;
                            }

                            let file = symbol
                                .filename()
                                .map(|f| f.display().to_string())
                                .unwrap_or_else(|| String::from("<unknown>"));

                            let line = symbol
                                .lineno()
                                .map(|l| l.to_string())
                                .unwrap_or_else(|| String::from("<unknown>"));

                            Some(format!(
                                "  {:>3}: {}\n             at {}:{}",
                                i, name, file, line
                            ))
                        })
                        .collect();

                    format!("\nstack backtrace:\n{}", frames.join("\n"))
                }
                _ => String::from(
                    "\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace",
                ),
            };

            let output = format!(
                "\nthread '{thread_name}' ({thread_id:?}) panicked at {location}:\n\
             {message}\
             {backtrace_str}",
            );

            let formatted = match LOGGER_MANAGER.try_read() {
                Ok(config) => {
                    let level_str = "ERROR";
                    config
                        .format
                        .replace(
                            "{{TIME}}",
                            &Local::now().format(config.time_format).to_string(),
                        )
                        .replace("{{PROJECT_NAME}}", config.project_name)
                        .replace("{{LOG_LEVEL}}", level_str)
                        .replace("{{MESSAGE}}", &output)
                }
                Err(_) => output.clone(),
            };

            PANIC_LOGGER.with(|logger| {
                if let Some(log_fn) = logger.borrow().as_ref() {
                    log_fn(&formatted);
                }
            });

            if let Ok(config) = LOGGER_MANAGER.try_read() {
                let dir = Path::new(config.log_file_path);
                if dir.exists() || fs::create_dir_all(dir).is_ok() {
                    let date = CREATE_TIME.get().map(|s| s.as_str()).unwrap_or("unknown");
                    let mut path = PathBuf::from(config.log_file_path);
                    path.push(
                        format!(
                            "{}_{}_{}_({}).log",
                            config.project_name,
                            API_NAME,
                            date,
                            COUNTER.load(Ordering::SeqCst)
                        )
                        .replace(" ", "_"),
                    );

                    if let Ok(mut file) =
                        fs::OpenOptions::new().append(true).create(true).open(&path)
                    {
                        let _ = writeln!(file, "{}", formatted);
                    }
                }
            }
        }));
    }

    pub fn debug_logging(message: &str) {
        Self::logging(LogLevel::Debug, message);
    }

    pub fn info_logging(message: &str) {
        Self::logging(LogLevel::Info, message);
    }

    pub fn warning_logging(message: &str) {
        Self::logging(LogLevel::Warning, message);
    }

    pub fn error_logging(message: &str) {
        Self::logging(LogLevel::Error, message);
    }

    pub fn logging(log_level: LogLevel, message: &str) {
        Self::logging_with_write(log_level, message, true);
    }

    pub fn logging_with_write(log_level: LogLevel, message: &str, write_log_file: bool) {
        let logging_text = Self::format_message(log_level, message);

        match log_level {
            LogLevel::Debug => debug_ln!("{}", logging_text),
            LogLevel::Info => info_ln!("{}", logging_text),
            LogLevel::Warning => warning_ln!("{}", logging_text),
            LogLevel::Error => error_ln!("{}", logging_text),
        }

        if write_log_file {
            if let Err(e) = Self::write_to_file(&logging_text) {
                Self::logging_with_write(
                    LogLevel::Error,
                    &format!("Unable to write to the log file. {}", e),
                    false,
                );
            }
        }
    }

    fn write_to_file(message: &str) -> Result<(), std::io::Error> {
        let config = LOGGER_MANAGER.read().unwrap();
        let dir = Path::new(config.log_file_path);

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let path = Self::get_log_file_path(&config)?;

        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)?;

        writeln!(file, "{}", message)?;
        Ok(())
    }

    fn get_log_file_path(config: &LoggerConfig) -> Result<PathBuf, std::io::Error> {
        let date = CREATE_TIME.get().map(|s| s.as_str()).unwrap_or("");

        let mut path = PathBuf::from(config.log_file_path);
        path.push(
            format!(
                "{}_{}_{}_({}).log",
                config.project_name,
                API_NAME,
                date,
                COUNTER.load(Ordering::SeqCst)
            )
            .replace(" ", "_"),
        );

        if !path.exists() {
            return Ok(path);
        }

        let metadata = fs::metadata(&path)?;
        if metadata.len() < config.max_file_size {
            Ok(path)
        } else {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            Self::get_log_file_path(config)
        }
    }

    fn format_message(level: LogLevel, message: &str) -> String {
        let config = LOGGER_MANAGER.read().unwrap();

        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };

        config
            .format
            .replace(
                "{{TIME}}",
                &Local::now().format(config.time_format).to_string(),
            )
            .replace("{{PROJECT_NAME}}", config.project_name)
            .replace("{{LOG_LEVEL}}", level_str)
            .replace("{{MESSAGE}}", message)
    }
}
