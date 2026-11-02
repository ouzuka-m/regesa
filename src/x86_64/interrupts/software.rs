//! Software interrupt handlers.
//!
//! Contains handlers for software-triggered interrupts (`INT n`)
//! used for internal kernel IPC, such as signalling that a command
//! line input buffer is ready to be processed.

use crate::prompt;
use x86_64::structures::idt::InterruptStackFrame;

/// Read-command software interrupt handler (Vector 48 / 0x30).
///
/// Triggered by the keyboard handler via `INT 0x30` when the user
/// presses Enter. Prints a new shell prompt and sends EOI.
pub extern "x86-interrupt" fn read_command(_: InterruptStackFrame) {
    prompt!();
}
