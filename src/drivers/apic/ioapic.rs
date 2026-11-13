//! I/O APIC (Input/Output Advanced Programmable Interrupt Controller) driver.
//!
//! Provides functions to query I/O APIC registers, map ISA hardware interrupt request (IRQ)
//! lines to Global System Interrupts (GSIs) via ACPI interrupt source overrides,
//! and program I/O Redirection Table entries (IOREDTBL).

use acpi::platform::interrupt::{InterruptSourceOverride, IoApic};
use core::ptr;
use spin::once::Once;
use x86_64::{PhysAddr, VirtAddr};

use crate::{
    drivers::{apic::lapic, input::ps2_keyboard},
    info,
    memory::address::{PhysExt, VirtExt},
    x86_64::idt::KEYBOARD_VECTOR,
};

/// Register Select offset relative to I/O APIC MMIO base.
const IOREGSEL: u64 = 0x00;
/// I/O Window offset relative to I/O APIC MMIO base.
const IOWIN: u64 = 0x10;

/// Register index for I/O APIC ID.
const IOAPICID: u32 = 0x00;
/// Register index for I/O APIC Version and Max Redirection Entries.
const IOAPICVER: u32 = 0x01;
/// Base register index for I/O Redirection Table entries.
const IOREDTBL_BASE: u32 = 0x10;

/// ISA IRQ line for PS/2 Keyboard.
const KEYBOARD_ISA: u8 = 1;

static BASE_ADDRESS: Once<VirtAddr> = Once::new();

/// Initializes the I/O APIC by configuring Redirection Table entries.
pub fn init(ioapic: &IoApic, overrides: &[InterruptSourceOverride]) {
    let base_address = PhysAddr::new(ioapic.address as u64).to_virt();

    BASE_ADDRESS.call_once(|| base_address);

    let id = ((read(IOAPICID) >> 24) & 0xFF) as u8;
    let (max_irqs, version) = {
        let value = read(IOAPICVER);

        ((value >> 16) as u8, value as u8)
    };

    info!(
        "I/O APIC initialized: ID = {}, Version = {:#x}, Max IRQs = {}",
        id,
        version,
        max_irqs + 1
    );

    let high_value = lapic::id() as u32; // Destination

    let keyboard_gsi = isa_to_gsi(KEYBOARD_ISA, overrides);

    let low = IOREDTBL_BASE + keyboard_gsi * 2;
    let high = low + 1;

    let low_value = KEYBOARD_VECTOR as u32;

    map_redtbl(low, high, low_value, high_value);

    // Fix keyboard sometimes "die" when you spamming keys on startup
    ps2_keyboard::clear_buffer();
}

/// Reads a 32-bit register value from te I/O APIC.
///
/// Writes the register index to `IOREGSEL` and reads the data from `IOWIN`.
fn read(reg: u32) -> u32 {
    let base_address = base_address();

    let ioregsel: *mut u32 = base_address.offset(IOREGSEL).as_mut_ptr();
    let iowin: *const u32 = base_address.offset(IOWIN).as_ptr();

    // SAFETY: The I/O APIC base address comes from the ACPI MADT and is
    // mapped via the HHDM. IOREGSEL and IOWIN are at fixed offsets 0x00
    // and 0x10 within the I/O APIC MMIO page.
    unsafe {
        ptr::write_volatile(ioregsel, reg);
        ptr::read_volatile(iowin)
    }
}

/// Writes a 32-bit value to an I/O APIC register.
///
/// Writes the register index to `IOREGSEL` followed by the value to `IOWIN`.
fn write(reg: u32, value: u32) {
    let base_address = base_address();

    let ioregsel: *mut u32 = (base_address + IOREGSEL).as_mut_ptr();
    let iowin: *mut u32 = (base_address + IOWIN).as_mut_ptr();

    // SAFETY: Same as `read` — valid I/O APIC MMIO addresses via HHDM.
    unsafe {
        ptr::write_volatile(ioregsel, reg);
        ptr::write_volatile(iowin, value);
    }
}

/// Configures a 64-bit Redirection Table entry (IOREDTBL) for an IRQ pin.
fn map_redtbl(low: u32, high: u32, low_value: u32, high_value: u32) {
    write(high, high_value);
    write(low, low_value);
}

/// Resolves an ISA IRQ line number to its corresponding Global System Interrupt (GSI).
///
/// Searches the ACPI interrupt source overrides slice. If an explicit override
/// exists for `isa_irq`, returns its `global_system_interrupt`; otherwise defaults to `isa_irq`.
fn isa_to_gsi(isa_irq: u8, overrides: &[InterruptSourceOverride]) -> u32 {
    overrides
        .iter()
        .find(|o| o.isa_source == isa_irq)
        .map(|o| o.global_system_interrupt)
        .unwrap_or(isa_irq as u32)
}

fn base_address() -> VirtAddr {
    *BASE_ADDRESS
        .get()
        .expect("IOAPIC address hasn't been initialized")
}
