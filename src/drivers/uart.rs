//! UART 16550 serial port driver.
//!
//! Initialises the first serial port (COM1, I/O port `0x3F8`) and
//! exposes it as a global mutex-protected static for kernel logging.

use core::sync::atomic::AtomicU8;

use spin::{Mutex, lazylock::LazyLock};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

const PORT: u16 = 0x3F8;

pub static UART_ERRORS: AtomicU8 = AtomicU8::new(0);
pub static DROPPED_LOGS: AtomicU8 = AtomicU8::new(0);

/// Global serial port instance used by the kernel logger.
///
/// Lazily initialised on first access. All writes are serialised
/// through the [`Mutex`].
pub static SERIAL: LazyLock<Mutex<Uart16550Tty<PioBackend>>> = LazyLock::new(|| {
    // SAFETY: Port 0x3F8 (COM1) is the standard first serial port on
    // x86 PCs. We initialise it exactly once via LazyLock, and no
    // other code accesses this port.
    unsafe {
        Mutex::new(
            Uart16550Tty::new_port(PORT, Config::default())
                .expect("Failed to initialize UART 16550"),
        )
    }
});
