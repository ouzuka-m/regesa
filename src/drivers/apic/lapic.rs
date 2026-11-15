//! Local APIC (Advanced Programmable Interrupt Controller) control module.
//!
//! Provides functionality to enable the Local APIC, send End of Interrupt (EOI)
//! signals, and query CPU APIC IDs via memory-mapped I/O (MMIO) registers.

use core::{ops::Add, ptr};

use spin::once::Once;
use x86_64::{PhysAddr, VirtAddr};

use crate::{debug, info, memory::address::PhysExt, x86_64::idt::SVR_VECTOR};

/// Offset for the Local APIC ID register.
const LAPIC_ID_REG: u64 = 0x20;

/// Offset for the End of Interrupt (EOI) register.
const LAPIC_EOI_REG: u64 = 0xB0;

/// Offset for the Spurious Interrupt Vector Register (SVR).
const LAPIC_SVR_REG: u64 = 0xF0;

/// Bit flag in SVR to enable the Local APIC.
const SVR_APIC_ENABLE: u32 = 1 << 8;

/// Global thread-safe singleton holding the initialized LAPIC virtual address.
static BASE_ADDRESS: Once<VirtAddr> = Once::new();

/// Enables the Local APIC at the given physical base address.
///
/// Converts the physical address to a virtual address, writes to the Spurious
/// Interrupt Vector Register (SVR at offset `0xF0`) to set the APIC enable bit
/// and set the spurious interrupt vector index ([`SVR_VECTOR`]), and stores the
/// global [`BASE_ADDRESS`] singleton.
///
/// # Parameters
/// - `lapic_phys_address`: Physical address of the Local APIC base registers.
pub fn init(lapic_phys_address: u64) {
    let base_address = PhysAddr::new(lapic_phys_address).to_virt();

    debug!(
        "Enabling Local APIC (Phys: {:#x}, Virt: {:#x})",
        lapic_phys_address, base_address
    );

    BASE_ADDRESS.call_once(|| base_address);

    let mut svr_value = read(LAPIC_SVR_REG);

    svr_value |= SVR_APIC_ENABLE;
    svr_value |= SVR_VECTOR as u32;

    write(LAPIC_SVR_REG, svr_value);

    info!("Local APIC enabled (ID: {})", id());
}

/// Sends an End of Interrupt (EOI) signal to the Local APIC.
///
/// Writing `0` to the EOI register (offset `0xB0`) signals to the Local APIC
/// that processing of the current interrupt has finished, allowing higher or
/// equal priority interrupts to be delivered.
pub fn eoi() {
    write(LAPIC_EOI_REG, 0);
}

/// Reads the Local APIC ID of the current processor.
///
/// Reads the value from the Local APIC ID register (offset `0x20`) and shifts
/// it appropriately.
///
/// # Returns
/// The APIC ID right-shifted by 24 bits as required by IOAPIC destination matching.
pub fn id() -> u8 {
    let value = read(LAPIC_ID_REG);
    (value >> 24) as u8
}

/// Configures a Local Vector Table (LVT) entry.
///
/// Writes `flags | vector` to the specified LVT register offset.
pub fn configure_lvt(reg: u64, vector: u8, flags: u32) {
    write(reg, flags | vector as u32);
}

/// Reads a 32-bit value from a Local APIC MMIO register.
fn read(offset: u64) -> u32 {
    let base_address = base_address();
    let src = base_address.add(offset).as_ptr::<u32>();

    // SAFETY: The base address originates from the ACPI MADT table and
    // is translated to a valid virtual address via the HHDM. The offsets
    // are well-known LAPIC register offsets within the 4 KiB MMIO page.
    unsafe { ptr::read_volatile(src) }
}

/// Writes a 32-bit value to a Local APIC MMIO register.
fn write(offset: u64, value: u32) {
    let base_address = base_address();
    let dst = base_address.add(offset).as_mut_ptr::<u32>();

    // SAFETY: Same as `read` — valid LAPIC MMIO region via HHDM.
    unsafe {
        ptr::write_volatile(dst, value);
    }
}

fn base_address() -> VirtAddr {
    *BASE_ADDRESS.get().expect("LAPIC hasn't been initialized")
}
