use core::fmt;

/// Errors that can occur when loading and spawning a new process/executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    // --- Scheduler / Process Manager ---
    /// The process table or scheduler queue is full. Cannot spawn more processes.
    ProcessLimitReached,

    // --- Filesystem / Loader ---
    /// The requested file or path does not exist.
    NotFound,

    /// The file exists but is not a valid executable format (bad magic, not ELF, etc.).
    NotExecutable,

    /// The file exists but there is no usable memory region to load it into.
    /// Usually means RAM is fragmented or all regions are reserved.
    OutOfMemory,

    /// Failed to map the executable's segments into virtual memory.
    /// Happens when page table allocation or permission setup fails.
    MemoryMapFailed,

    /// A program segment (e.g., PT_LOAD) has invalid offset, size, or alignment.
    InvalidSegment,

    MissingProgramHeaders,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessLimitReached => write!(f, "process limit reached: scheduler is full"),
            Self::NotFound => write!(f, "file not found"),
            Self::NotExecutable => write!(f, "file is not a valid executable"),
            Self::OutOfMemory => write!(f, "not enough usable memory to load executable"),
            Self::MemoryMapFailed => write!(f, "failed to map executable into memory"),
            Self::InvalidSegment => write!(f, "executable contains invalid segment data"),
            Self::MissingProgramHeaders => write!(f, "executable doesn't have program headers"),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
