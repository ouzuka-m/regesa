//! Log severity levels.
//!
//! Defines the [`Level`] enum used to filter log output by severity.

use core::sync::atomic::{AtomicU8, Ordering};

pub static LOG_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl Level {
    pub fn as_str(&self) -> &str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

pub fn set_level(level: Level) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn enabled(level: Level) -> bool {
    level as u8 >= LOG_LEVEL.load(Ordering::Relaxed)
}
