//! Task termination and exit routine.
//!
//! Serves as the exit gateway for the scheduler when a task finishes execution.

use x86_64::instructions::interrupts;

/// Exit gateway routine for terminated tasks in the scheduler.
///
/// Halts the CPU while awaiting interrupts once a task has finished its execution.
pub fn exit() -> ! {
    loop {
        interrupts::enable_and_hlt();
    }
}
