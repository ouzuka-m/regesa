//! Kernel logging infrastructure.
//!
//! Provides the internal [`_log`] function and the public [`debug!`],
//! [`info!`], [`warn!`], and [`error!`] macros that write timestamped,
//! level-tagged messages to the UART serial port.

use core::{
    fmt::{Arguments, Write},
    sync::atomic::Ordering,
};
use x86_64::instructions::interrupts;

use crate::{
    drivers::uart::{DROPPED_LOGS, SERIAL, UART_ERRORS},
    log::level::{self, Level},
};

/// Internal logging function used by the logging macros.
pub fn _log(level: Level, args: Arguments) {
    if !level::enabled(level) {
        return;
    }

    let args = format_args!("[{}] {}\n", level.as_str(), args);

    interrupts::without_interrupts(|| {
        if let Some(mut serial) = SERIAL.try_lock() {
            if serial.write_fmt(args).is_err() {
                UART_ERRORS.fetch_add(1, Ordering::Relaxed);
            };
        } else {
            DROPPED_LOGS.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// Logs a message at the debug level.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log::macros::_log($crate::log::level::Level::Debug, format_args!($($arg)*))
    };
}

/// Logs a message at the info level.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log::macros::_log($crate::log::level::Level::Info, format_args!($($arg)*))
    };
}

/// Logs a message at the warn level.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log::macros::_log($crate::log::level::Level::Warn, format_args!($($arg)*))
    };
}

/// Logs a message at the error level.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log::macros::_log($crate::log::level::Level::Error, format_args!($($arg)*))
    };
}
