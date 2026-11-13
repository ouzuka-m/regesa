//! Idle task routine for the scheduler.
//!
//! Provides the idle loop that keeps the CPU in a low-power halting state when no
//! other tasks are runnable, preventing excessive CPU spin. This task is designed
//! to reside at index 0 (the first index) of the scheduler task array.

use x86_64::instructions::interrupts;

/// Idle task loop.
///
/// Keeps the CPU halted while interrupts are enabled to wait for work without busy-waiting.
/// Designed to always occupy index 0 in the scheduler task array.
pub fn idle() -> ! {
    // TODO: deallocate memory and clear the dead tasks
    loop {
        interrupts::enable_and_hlt();
    }
}
