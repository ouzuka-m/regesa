//! # Stack Constants and IST Indices
//!
//! Defines the default allocation size for kernel exception stacks and specifies
//! the Interrupt Stack Table (IST) index mappings used by the TSS and IDT.

use x86_64::structures::paging::{PageSize, Size4KiB};

/// Size of each dedicated exception stack in bytes (8 KiB).
pub const STACK_SIZE: usize = (Size4KiB::SIZE * 2) as usize; // 8 KiB

/// Internal wrapper representing a 16-byte aligned stack memory buffer.
#[repr(C, align(16))]
pub struct Stack([u8; STACK_SIZE]);

pub static mut DF_STACK: Stack = Stack([0; STACK_SIZE]);
pub static mut NMI_STACK: Stack = Stack([0; STACK_SIZE]);
pub static mut MCE_STACK: Stack = Stack([0; STACK_SIZE]);

pub static mut KERNEL_STACK: Stack = Stack([0; STACK_SIZE]);

/// IST index for the Double Fault (#DF) exception stack.
pub const DF_INDEX: u16 = 0;

/// IST index for the Non-Maskable Interrupt (#NMI) stack.
pub const NMI_INDEX: u16 = 1;

/// IST index for the Machine Check Exception (#MC) stack.
pub const MCE_INDEX: u16 = 2;

/// Kernel stack for RSP0
pub const KERNEL_INDEX: u16 = 0;
