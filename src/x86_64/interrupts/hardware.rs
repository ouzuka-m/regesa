//! Hardware interrupt handlers (IRQs).
//!
//! This module contains interrupt service routines (ISRs) for handling
//! hardware-level interrupts delivered via the APIC, such as spurious
//! interrupts, APIC timer ticks, and PS/2 keyboard inputs.

use alloc::string::String;
use pc_keyboard::DecodedKey;
use spin::mutex::Mutex;
use x86_64::{instructions::interrupts, structures::idt::InterruptStackFrame};

use crate::{
    config::CONFIG,
    drivers::{apic::lapic, input::ps2_keyboard, tsc_deadline},
    print, scheduler, warn,
    x86_64::idt::READ_COMMAND_VECTOR,
};

pub static INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());

/// Spurious vector interrupt handler (Vector 255 / 0xFF).
///
/// Spurious interrupts occur when an interrupt signal is dropped before the APIC
/// can process it. According to the Intel APIC specification, spurious interrupts
/// do not require an End of Interrupt (EOI) signal.
pub extern "x86-interrupt" fn spurious_vector_interrupt(_: InterruptStackFrame) {
    warn!("Spurious vector interrupt (SVR) triggered");
}

/// Timer interrupt handler (Vector 32 / 0x20).
///
/// Triggered periodically by the APIC timer or PIT to drive OS scheduling
/// and timekeeping tasks. Sends an EOI signal to the Local APIC upon completion.
pub extern "x86-interrupt" fn timer(_: InterruptStackFrame) {
    tsc_deadline::arm(CONFIG.tick_interval());

    lapic::eoi();

    scheduler::schedule();
}

/// PS/2 Keyboard interrupt handler (Vector 33 / 0x21).
///
/// Reads raw scancodes from I/O port `0x60`, decodes keypress events using the
/// layout parser, logs the key, and issues an EOI to the Local APIC.
pub extern "x86-interrupt" fn keyboard(_: InterruptStackFrame) {
    let scancode = ps2_keyboard::read();

    if let Some(key) = ps2_keyboard::decode(scancode)
        && let DecodedKey::Unicode(c) = key
    {
        match c {
            '\u{8}' => {
                if INPUT_BUFFER.lock().pop().is_some() {
                    print!(c);
                }
            }

            '\n' => {
                print!(c);

                // SAFETY: INT 0x30 triggers the read_command software
                // interrupt handler registered in the IDT. The handler
                // only prints a prompt and sends EOI.
                unsafe {
                    interrupts::software_interrupt::<READ_COMMAND_VECTOR>();
                }

                INPUT_BUFFER.lock().clear();
            }

            _ => {
                print!(c);

                INPUT_BUFFER.lock().push(c);
            }
        }
    }

    lapic::eoi();
}
