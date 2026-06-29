use std::io::Write;
use std::{fs, panic};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use backtrace::Backtrace;
use chrono::Local;
use crate::{debug_ln, info_ln, warning_ln, error_ln};
use crate::vitreous_rs::{API_NAME, DEFAULT_PROJECT_NAME};

static IS_FIRST_WRITE: AtomicBool = AtomicBool::new(true);

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

pub struct LoggerManager {
    project_name:  &'static str,
    format:        &'static str,
    time_format:   &'static str,
    log_file_path: &'static str,
    max_file_size: u64,
    create_time:   OnceLock<String>,
}

impl LoggerManager {
    pub const fn new(
        project_name:  &'static str,
        format:        &'static str,
        time_format:   &'static str,
        log_file_path: &'static str,
        max_file_size: u64,
    ) -> LoggerManager {
        Self {
            project_name,
            format,
            time_format,
            log_file_path,
            max_file_size,
            create_time: OnceLock::new(),
        }
    }

    pub const fn default_instance() -> Self {
        Self::new(
            DEFAULT_PROJECT_NAME,
            "[{{TIME}}][{{PROJECT_NAME}}][{{LOG_LEVEL}}] {{MESSAGE}}",
            "%Y/%m/%d %H:%M:%S",
            "logs",
            1024 * 1024 * 10,
        )
    }

    pub fn init(&'static self) {
        self.create_time.get_or_init(|| {
            Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
        });

        self.setup_panic_hook();
    }

    fn get_create_time(&self) -> &str {
        self.create_time.get_or_init(|| {
            Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
        })
    }

    pub fn setup_panic_hook(&'static self) {
        PANIC_LOGGER.with(|logger| {
            *logger.borrow_mut() = Some(Box::new(move |message| {
                self.error_logging(message);
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

            let thread      = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");
            let thread_id   = std::thread::current().id();

            let backtrace_str = match std::env::var("RUST_BACKTRACE").as_deref() {
                Ok("full") => {
                    let bt = Backtrace::new();
                    format!("\nstack backtrace:\n{:?}", bt)
                }
                Ok("1") => {
                    let bt = Backtrace::new();
                    let frames: Vec<String> = bt.frames()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, frame)| {
                            let symbol = frame.symbols().first()?;
                            let name = symbol.name()
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| String::from("<unknown>"));

                            if name.contains("backtrace")
                                || name.contains("panic")
                                || name.contains("__rust")
                                || name.contains("setup_panic_hook")
                            {
                                return None;
                            }

                            let file = symbol.filename()
                                .map(|f| f.display().to_string())
                                .unwrap_or_else(|| String::from("<unknown>"));

                            let line = symbol.lineno()
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
                    "\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
                ),
            };

            let output = format!(
                "\nthread '{thread_name}' ({thread_id:?}) panicked at {location}:\n\
                 {message}\
                 {backtrace_str}",
            );

            PANIC_LOGGER.with(|logger| {
                if let Some(log_fn) = logger.borrow().as_ref() {
                    log_fn(&output);
                }
            });
        }));
    }

    pub fn debug_logging(&self, message: &str) {
        self.logging(LogLevel::Debug, message);
    }

    pub fn info_logging(&self, message: &str) {
        self.logging(LogLevel::Info, message);
    }

    pub fn warning_logging(&self, message: &str) {
        self.logging(LogLevel::Warning, message);
    }

    pub fn error_logging(&self, message: &str) {
        self.logging(LogLevel::Error, message);
    }

    pub fn logging(&self, log_level: LogLevel, message: &str) {
        self.logging_with_write(log_level, message, true);
    }

    pub fn logging_with_write(&self, log_level: LogLevel, message: &str, write_log_file: bool) {
        let logging_text = self.format_message(log_level, message);

        match log_level {
            LogLevel::Debug   => debug_ln!("{}", logging_text),
            LogLevel::Info    => info_ln!("{}", logging_text),
            LogLevel::Warning => warning_ln!("{}", logging_text),
            LogLevel::Error   => error_ln!("{}", logging_text),
        }

        if write_log_file {
            if let Err(e) = self.write_to_file(&logging_text) {
                self.logging_with_write(
                    LogLevel::Error,
                    &format!("Unable to write to the log file. {}", e),
                    false,
                );
            }
        }
    }

    fn write_to_file(&self, message: &str) -> Result<(), std::io::Error> {
        let dir = Path::new(self.log_file_path);

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let path = self.get_log_file_path()?;

        let mut file = if IS_FIRST_WRITE.compare_exchange(
            true, false,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).is_ok() {
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?
        } else {
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)?
        };

        writeln!(file, "{}", message)?;
        Ok(())
    }

    fn get_log_file_path(&self) -> Result<PathBuf, std::io::Error> {
        let date = self.get_create_time();
        let mut number = 1;

        loop {
            let mut path = PathBuf::from(self.log_file_path);
            path.push(
                format!("{}_{}_{}_({}).log",
                        self.project_name,
                        API_NAME,
                        date,
                        number
                ).replace(" ", "_")
            );

            if !path.exists() {
                return Ok(path);
            }

            let metadata = fs::metadata(&path)?;
            if metadata.len() < self.max_file_size {
                return Ok(path);
            }

            number += 1;
        }
    }

    fn format_message(&self, level: LogLevel, message: &str) -> String {
        let level_str = match level {
            LogLevel::Debug   => "DEBUG",
            LogLevel::Info    => "INFO ",
            LogLevel::Warning => "WARN ",
            LogLevel::Error   => "ERROR",
        };

        self.format
            .replace("{{TIME}}",         &Local::now().format(self.time_format).to_string())
            .replace("{{PROJECT_NAME}}", self.project_name)
            .replace("{{LOG_LEVEL}}",    level_str)
            .replace("{{MESSAGE}}",      message)
    }
}