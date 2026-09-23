//! a lightweight logger with stdout and file output options.

use std::{
    fmt::Arguments,
    fs::{File, OpenOptions},
    io::{LineWriter, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use parking_lot::Mutex;

/// configuration options for the logger.
#[derive(Debug, Clone)]
pub struct LoggerOptions {
    /// the logging level.
    pub level: Level,
    /// optional file path to write logs to.
    pub file: Option<PathBuf>,
    /// whether to truncate the log file, if present.
    pub truncate: bool,
    /// whether to write logs to stdout.
    pub stdout: bool,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self {
            level: Level::Info,
            file: None,
            truncate: false,
            stdout: true,
        }
    }
}

impl LoggerOptions {
    /// sets the logging level.
    #[must_use]
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// sets the log file path.
    #[must_use]
    pub fn file(mut self, file: impl AsRef<Path>) -> Self {
        self.file = Some(file.as_ref().to_path_buf());
        self
    }

    /// enables or disables log file truncation.
    #[must_use]
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// enables or disables stdout logging.
    #[must_use]
    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }
}

/// initializes the logger.
/// should only be called once.
pub fn init(options: LoggerOptions, func: LogFunction) -> std::io::Result<()> {
    let file = match &options.file {
        Some(path) => Some(LineWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(options.truncate)
                .append(!options.truncate)
                .open(path)?,
        )),
        None => None,
    };

    LOGGER.get_or_init(|| {
        Mutex::new(Logger {
            func,
            file,
            options,
        })
    });

    Ok(())
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Debug => "DEBUG",
                Self::Info => "INFO",
                Self::Warn => "WARN",
                Self::Error => "ERROR",
            }
        )
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($args:tt)+) => {
        $crate::log(
            $level,
            file!(),
            line!(),
            format_args!($($args)+),
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {
        $crate::log!($crate::Level::Debug, $($args)+)
    };
}

#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {
        $crate::log!($crate::Level::Info, $($args)+)
    };
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {
        $crate::log!($crate::Level::Warn, $($args)+)
    };
}

#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {
        $crate::log!($crate::Level::Error, $($args)+)
    };
}

pub struct Location {
    pub file: &'static str,
    pub line: u32,
}

/// a log record containing level, location, and formatted arguments.
pub struct Record<'a> {
    pub level: Level,
    pub location: Location,
    pub args: Arguments<'a>,
}

pub type LogFunction = fn(writer: &mut dyn Write, record: &Record) -> std::io::Result<()>;

pub fn log(level: Level, file: &'static str, line: u32, args: Arguments) {
    if let Some(logger) = LOGGER.get() {
        logger.lock().log(Record {
            level,
            location: Location { file, line },
            args,
        });
    }
}

static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

struct Logger {
    func: LogFunction,
    file: Option<LineWriter<File>>,
    options: LoggerOptions,
}

impl Logger {
    fn log(&mut self, record: Record) {
        if record.level < self.options.level {
            return;
        }

        if let Some(file) = &mut self.file {
            let _ = (self.func)(file, &record);
        }

        if self.options.stdout {
            let _ = (self.func)(&mut std::io::stdout().lock(), &record);
        }
    }
}
