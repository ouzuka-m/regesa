//! Kernel runtime configuration.
//!
//! Parses the kernel command line supplied by the bootloader and
//! initializes runtime configuration with validated values.

pub mod tick;

use crate::{
    boot::info::CMDLINE,
    log::level::{self, Level},
};
use spin::lazylock::LazyLock;
use tick::TickInterval;

static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::parse(*CMDLINE));

#[derive(Debug)]
pub struct Config {
    log_level: Level,
    tick_interval: TickInterval,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: Level::Info,
            tick_interval: TickInterval::Ms100,
        }
    }
}

impl Config {
    fn parse(cmdline: &str) -> Self {
        let mut config = Self::default();

        for arg in cmdline.split_whitespace() {
            let Some((key, value)) = arg.split_once('=') else {
                continue;
            };

            match key {
                "log_level" => {
                    config.log_level = Self::parse_log_level(value);
                }
                "tick_ms" => {
                    config.tick_interval = TickInterval::parse(value);
                }
                _ => {}
            }
        }

        config
    }

    fn parse_log_level(value: &str) -> Level {
        match value {
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" => Level::Warn,
            "error" => Level::Error,
            _ => panic!("Invalid log_level value. Available option: debug, info, warn, error"),
        }
    }
}

pub(crate) fn init() {
    let config = &*CONFIG;

    level::set_level(config.log_level);
}

// pub fn log_level() -> Level {
//     CONFIG.log_level
// }

pub fn tick_interval() -> TickInterval {
    CONFIG.tick_interval
}
