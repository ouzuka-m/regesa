//! x86_64 architecture-specific structures and initialization.
//!
//! This module contains hardware-level abstractions and setup for the x86_64
//! platform, including the Global Descriptor Table (GDT), Interrupt Descriptor Table (IDT),
//! Task State Segment (TSS), interrupt handling, and SSE enablement.

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod ist;
pub mod sse;
pub mod tss;
