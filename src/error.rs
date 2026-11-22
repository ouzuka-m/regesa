use core::fmt;

use elf::{ParseError, file::Class};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Size4KiB, mapper::MapToError},
};

/// Errors that can occur when loading and spawning a new process/executable.
#[derive(Debug)]
pub enum Error {
    // --- Scheduler / Process Manager ---
    /// The process table or scheduler queue is full. Cannot spawn more processes.
    ProcessLimitReached,

    // --- Filesystem / Loader ---
    /// The requested file or path does not exist.
    NotFound,

    /// ELF isn't a valid executable.
    ELFParser { context: ParseError },

    /// ELF isn't supported format.
    UnsupportedElfFormat {
        e_type: u16,
        e_class: Class,
        e_machine: u16,
    },

    /// ELF doesn't have program headers.
    MissingProgramHeaders,

    // -- Memory --
    /// The file exists but there is no usable memory region to load it into.
    /// Usually means RAM is fragmented or all regions are reserved.
    OutOfMemory,

    /// Failed to map the executable's segments into virtual memory.
    /// Happens when page table allocation or permission setup fails.
    MemoryMapFailed {
        page_addr: VirtAddr,
        frame_addr: PhysAddr,
        context: MapToError<Size4KiB>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessLimitReached => write!(f, "Process limit reached"),
            Self::NotFound => write!(f, "File not found"),
            Self::ELFParser { context } => write!(f, "File is not a valid executable: {context}"),
            Self::UnsupportedElfFormat {
                e_type,
                e_class,
                e_machine,
            } => write!(
                f,
                "Unsupported ELF Format: e_type={:?}, class={:?}, e_machine={:?}",
                e_type, e_class, e_machine
            ),
            Self::MissingProgramHeaders => write!(f, "Executable doesn't have program headers"),
            Self::OutOfMemory => write!(f, "Not enough usable memory to load executable"),
            Self::MemoryMapFailed {
                page_addr,
                frame_addr,
                context,
            } => write!(
                f,
                "Failed to map page {:?} to frame {:?}: {:?}",
                page_addr, frame_addr, context
            ),
        }
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::ELFParser { context: value }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
