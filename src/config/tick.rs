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

    pub fn parse(value: &str) -> Self {
        match value {
            "100" => Self::Ms100,
            "500" => Self::Ms500,
            "1000" => Self::Ms1000,
            _ => panic!("Invalid tick_ms value. Available option: 100, 500, 1000"),
        }
    }
}
