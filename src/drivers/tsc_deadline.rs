//! TSC-Deadline timer driver.
//!
//! Configures the Local APIC timer in TSC-Deadline mode, calibrates the
//! TSC frequency against the HPET, and provides an [`arm`] function to
//! schedule one-shot timer interrupts at millisecond granularity.

use core::arch::x86_64::__cpuid;

use spin::once::Once;
use x86_64::registers::model_specific::Msr;

use crate::{
    config::tick::TickInterval,
    drivers::{hpet, lapic},
    x86_64::idt::TIMER_VECTOR,
};

const LVT: u64 = 0x320;
const TSC_DEADLINE_MODE: u32 = 2 << 17;
const CALIBRATION_MS: u64 = 10;

const IA32_TSC_DEADLINE: u32 = 0x6E0;

static TICKS_PER_MS: Once<u64> = Once::new();

/// Initialises the TSC-Deadline timer.
///
/// Verifies CPU support, configures the LVT timer entry in TSC-Deadline
/// mode, calibrates the TSC tick rate against the HPET, and arms the
/// first timer interrupt.
///
/// # Panics
/// Panics if the CPU does not support TSC-Deadline mode.
pub fn init() {
    assert!(supports(), "CPU doesn't support TSC-Deadline");

    lapic::configure_lvt(LVT, TIMER_VECTOR, TSC_DEADLINE_MODE);

    // SAFETY: _rdtsc reads the Time Stamp Counter, a non-destructive
    // operation available on all x86_64 CPUs.
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    hpet::wait_ms(CALIBRATION_MS);
    let end = unsafe { core::arch::x86_64::_rdtsc() };

    let ticks = (end - start) / CALIBRATION_MS;
    TICKS_PER_MS.call_once(|| ticks);

    crate::info!("TSC-Deadline timer initialized ({} ticks/ms)", ticks);

    wrmsr(1);
}

/// Arms the TSC-Deadline timer to fire.
pub fn arm(tick: TickInterval) {
    let ms = tick.as_millis();

    // SAFETY: _rdtsc is a non-destructive read of the TSC register.
    let now = unsafe { core::arch::x86_64::_rdtsc() };
    let ticks_per_ms = ticks_per_ms();

    let delta = ticks_per_ms.checked_mul(ms).expect("TSC delay overflow");
    let deadline = now.checked_add(delta).expect("TSC deadline overflow");

    wrmsr(deadline);
}

/// Returns the calibrated TSC ticks-per-millisecond value.
pub fn ticks_per_ms() -> u64 {
    *TICKS_PER_MS.get().expect("Ticks per MS haven't calculated")
}

/// Writes a value to the IA32_TSC_DEADLINE MSR.
fn wrmsr(value: u64) {
    // SAFETY: Writing the IA32_TSC_DEADLINE MSR (0x6E0) programs the
    // next TSC-Deadline timer interrupt. The caller is responsible for
    // providing a valid future TSC value.
    unsafe { Msr::new(IA32_TSC_DEADLINE).write(value) };
}

/// Checks whether the CPU supports TSC-Deadline mode (CPUID.01H:ECX bit 24).
fn supports() -> bool {
    // SAFETY: CPUID leaf 1 is guaranteed to exist on all x86_64 processors.
    let cpuid = __cpuid(1);
    (cpuid.ecx & (1 << 24)) != 0
}
