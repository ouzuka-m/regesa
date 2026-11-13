//! Streaming SIMD Extensions (SSE) initialization.
//!
//! This module provides functionality to enable SSE instructions on x86_64
//! by configuring the CR0 and CR4 control registers.

use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

pub fn init() {
    let mut cr0 = Cr0::read();
    let mut cr4 = Cr4::read();

    cr0 &= !Cr0Flags::EMULATE_COPROCESSOR;
    cr0 |= Cr0Flags::MONITOR_COPROCESSOR;

    unsafe { Cr0::write(cr0) };

    cr4 |= Cr4Flags::OSFXSR;
    cr4 |= Cr4Flags::OSXMMEXCPT_ENABLE;

    unsafe { Cr4::write(cr4) };
}
