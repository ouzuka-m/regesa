//! High Precision Event Timer (HPET) driver.
//!
//! Provides functions to initialise the HPET, read its main counter, and
//! perform busy-wait delays with nanosecond and millisecond granularity.
//! Used during TSC calibration and any early-boot timing requirements.

use core::{ops::Add, ptr};

use acpi::HpetInfo;
use spin::once::Once;
use x86_64::{PhysAddr, VirtAddr};

use crate::memory::address::PhysExt;

const CAP_ID: u64 = 0x000;
const CONFIG: u64 = 0x010;
const MAIN_COUNTER: u64 = 0x0F0;

static BASE_ADDRESS: Once<VirtAddr> = Once::new();
static PERIOD_FS: Once<u64> = Once::new();

/// Initialises the HPET from ACPI-provided information.
///
/// Reads the counter tick period from the capability register, validates
/// it, and enables the main counter.
///
/// # Panics
/// Panics if the HPET counter tick period is zero or exceeds 100 ms.
pub fn init(hpet_info: &HpetInfo) {
    let base_address = PhysAddr::new(hpet_info.base_address as u64).to_virt();

    BASE_ADDRESS.call_once(|| base_address);

    let period_fs = read(CAP_ID) >> 32;
    if period_fs == 0 || period_fs > 0x05F5E100 {
        panic!("Invalid HPET main counter tick");
    }

    PERIOD_FS.call_once(|| period_fs);

    // Enable CNF
    write(CONFIG, 0x1);

    crate::info!("HPET initialized (period: {} fs)", period_fs);
}

pub fn wait_ns(ns: u64) {
    let period_fs = period_fs();

    let ticks = ns.checked_mul(1_000_000).expect("wait_ns overflow") / period_fs;
    let target = counter() + ticks;
    while counter() < target {
        core::hint::spin_loop();
    }
}

pub fn wait_ms(ms: u64) {
    wait_ns(ms * 1_000_000);
}

pub fn counter() -> u64 {
    read(MAIN_COUNTER)
}

/// Reads a 64-bit value from an HPET MMIO register.
fn read(offset: u64) -> u64 {
    let base_address = base_address();
    let src = base_address.add(offset).as_ptr::<u64>();

    // SAFETY: The base address is set from the ACPI HPET table during init
    // and the register offsets are within the HPET MMIO region.
    unsafe { ptr::read_volatile(src) }
}

/// Writes a 64-bit value to an HPET MMIO register.
fn write(offset: u64, value: u64) {
    let base_address = base_address();
    let dst = base_address.add(offset).as_mut_ptr::<u64>();

    // SAFETY: The base address is set from the ACPI HPET table during init
    // and the register offsets are within the HPET MMIO region.
    unsafe { ptr::write_volatile(dst, value) }
}

fn base_address() -> VirtAddr {
    *BASE_ADDRESS
        .get()
        .expect("HPET address hasn't been initialized")
}

fn period_fs() -> u64 {
    *PERIOD_FS
        .get()
        .expect("Counter tick period hasn't been initialized")
}
