//! Kernel runtime configuration.
//!
//! Parses the kernel command line supplied by the bootloader and
//! initializes runtime configuration with validated values.

use spin::lazylock::LazyLock;

use crate::{
    boot::info::CMDLINE,
    log::level::{self, Level},
};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::parse(*CMDLINE));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickInterval {
    Ms100,
    Ms500,
    Ms1000,
}

impl TickInterval {
    pub const fn as_millis(self) -> u64 {
        match self {
            Self::Ms100 => 100,
            Self::Ms500 => 500,
            Self::Ms1000 => 1000,
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "100" => Self::Ms100,
            "500" => Self::Ms500,
            "1000" => Self::Ms1000,
            _ => panic!("Invalid tick_ms value. Available option: 100, 500, 1000"),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    tick_interval: TickInterval,
}

impl Default for Config {
    fn default() -> Self {
        Self {
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
                    let level = Self::parse_log_level(value);
                    level::set_level(level);
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

    pub const fn tick_interval(&self) -> TickInterval {
        self.tick_interval
    }
}
