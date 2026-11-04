//! # Regesa Kernel Entry Point
//!
//! This module serves as the primary entry point for the Regesa operating system kernel.
//! It disables interrupts during early boot, initializes essential core subsystems
//! (GDT, IDT, page frame allocation, virtual memory mapping, global heap allocator, and ACPI),
//! and transitions the CPU into an power-efficient idle loop waiting for interrupts.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![deny(clippy::unwrap_used)]

extern crate alloc;

mod boot;
mod config;
mod display;
mod drivers;
mod initramfs;
mod log;
mod memory;
mod qemu;
mod scheduler;
mod x86_64;

use ::x86_64::instructions::{self, interrupts};
use core::panic::PanicInfo;

use self::{
    display::greet,
    initramfs::TAR,
    memory::heap_allocator,
    x86_64::{gdt, idt},
};

/// Kernel entry point called by the Limine bootloader.
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    // Disable CPU hardware interrupts during critical boot sequence
    instructions::interrupts::disable();

    config::init();

    info!("Regesa Kernel starting...");

    // Load Global Descriptor Table (GDT) and Task State Segment (TSS)
    gdt::init();

    // Load Interrupt Descriptor Table (IDT) with exception & IRQ handlers
    idt::init();

    // Set up the global buddy system heap allocator
    heap_allocator::init();

    // Initialize all subsystems
    drivers::init();

    // Greet to the screen
    greet::welcome();

    interrupts::enable();

    info!("Kernel initialized successfully");

    qemu::exit::success();

    info!("Executing `/bin/init.bin`");
    if let Err(error) = TAR.spawn("/bin/init.bin") {
        panic!("Failed to execute `/bin/init.bin`: {error:#?}");
    };

    scheduler::activate();

    prompt!();

    loop {
        instructions::hlt();
    }
}

/// Custom panic handler for bare-metal `#![no_std]` environment.
///
/// Invoked automatically when a Rust `panic!` macro is triggered anywhere in the kernel.
/// It disables CPU interrupts, logs the panic information over the UART serial line,
/// and permanently halts the CPU in a `hlt` loop.
///
/// # Parameters
/// - `info`: Information regarding the location and message of the panic.
///
/// # Divergence
/// This function never returns (`-> !`).
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further interrupt processing during a fatal panic
    instructions::interrupts::disable();

    // Output formatted panic details to serial logger
    if let Some(location) = info.location() {
        error!(
            "KERNEL PANIC: {}\n  at {}:{}:{}\n",
            info.message(),
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        error!("KERNEL PANIC: {}\n", info.message());
    }

    qemu::exit::failure();

    // Enter infinite halt loop
    loop {
        instructions::hlt();
    }
}
