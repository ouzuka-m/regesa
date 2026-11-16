//! Boot information accessors.
//!
//! Lazily-initialized statics that extract and cache responses from the
//! Limine boot protocol requests, providing ergonomic kernel-wide access
//! to physical memory layout, HHDM offset, RSDP, framebuffer, loaded
//! modules, and the kernel command line.

use limine::{
    memmap::Entry,
    request::{ModulesRespData, Response},
};
use spin::{lazylock::LazyLock, mutex::Mutex};
use x86_64::VirtAddr;

use crate::{boot::requests, display::framebuffer::FrameBuffer};

/// Virtual base address offset of the Higher-Half Direct Mapping (HHDM).
///
/// The bootloader maps all physical memory directly at this virtual address offset.
///
/// # Panics
/// Panics if the bootloader fails to return a valid HHDM response.
pub static HHDM: LazyLock<VirtAddr> = LazyLock::new(|| {
    let hhdm_response = requests::HHDM_REQUEST
        .response()
        .expect("Failed to receive HHDM response from bootloader");

    VirtAddr::new(hhdm_response.offset)
});

/// Physical memory map entries provided by the bootloader.
///
/// Contains information about each physical memory region, including base address,
/// length, and region type (usable RAM, reserved, ACPI, etc.).
///
/// # Panics
/// Panics if the bootloader fails to return a valid memory map response.
pub static ENTRIES: LazyLock<&'static [&'static Entry]> = LazyLock::new(|| {
    requests::MEMMAP_REQUEST
        .response()
        .expect("Failed to receive memory map response from bootloader")
        .entries()
});

/// Virtual address of the ACPI Root System Description Pointer (RSDP).
///
/// Used by the ACPI subsystem to discover system description tables (e.g., MADT, FADT).
///
/// # Panics
/// Panics if the bootloader fails to return an RSDP response.
pub static RSDP: LazyLock<VirtAddr> = LazyLock::new(|| {
    let rsdp_address = requests::RSDP_REQUEST
        .response()
        .expect("Failed to receive RSDP response from bootloader")
        .address;

    VirtAddr::new(rsdp_address as u64)
});

/// Global framebuffer instance wrapped in a mutex for display rendering.
///
/// Initialized with the dimensions, pitch, and pixel buffer pointer obtained
/// from the bootloader's first framebuffer response.
///
/// # Panics
/// Panics if the bootloader fails to provide a framebuffer response or
/// if no framebuffer is present.
pub static FRAMEBUFFER: LazyLock<Mutex<FrameBuffer>> = LazyLock::new(|| {
    let framebuffers = requests::FRAMEBUFFER_REQUEST
        .response()
        .expect("Failed to receive framebuffers from bootloader")
        .framebuffers();

    let framebuffer = framebuffers
        .first()
        .expect("No framebuffer provided by bootloader");

    Mutex::new(FrameBuffer::new(
        framebuffer.address() as *mut u32,
        framebuffer.width as usize,
        framebuffer.height as usize,
        (framebuffer.pitch / 4) as usize,
    ))
});

/// Loaded bootloader modules response.
///
/// Provides access to external files loaded into memory alongside the kernel,
/// such as the initial ramdisk (initramfs).
///
/// # Panics
/// Panics if the bootloader fails to provide a modules response.
pub static MODULES: LazyLock<&'static Response<ModulesRespData>> = LazyLock::new(|| {
    requests::MODULES_REQUEST
        .response()
        .expect("Failed to get modules response from bootloader")
});

/// Kernel command-line parameters passed by the bootloader.
///
/// Returns an empty string if no command line was specified.
pub static CMDLINE: LazyLock<&'static str> =
    LazyLock::new(|| match requests::CMDLINE_REQUEST.response() {
        Some(response) => response.cmdline(),
        None => "",
    });
