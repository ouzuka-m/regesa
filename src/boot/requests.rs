//! Limine boot protocol requests.
//!
//! Declares the static request objects that the Limine bootloader scans for
//! during early boot. Each request is populated with a response by the
//! bootloader before control is transferred to the kernel entry point.

use limine::request::{
    ExecutableCmdlineRequest, FramebufferRequest, HhdmRequest, MemmapRequest, ModulesRequest,
    RsdpRequest,
};

/// Higher-Half Direct Map (HHDM) request.
///
/// Requests the bootloader to provide the virtual address offset where all physical
/// memory is directly mapped into the higher-half of the kernel address space.
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Memory map request.
///
/// Requests a detailed physical memory map from the bootloader, identifying usable
/// RAM, reserved regions, ACPI tables, and bootloader-reclaimable frames.
pub static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

/// ACPI RSDP (Root System Description Pointer) request.
///
/// Requests the physical address of the ACPI RSDP descriptor structure, used
/// for discovering ACPI tables (MADT, FADT, etc.).
pub static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// Linear framebuffer request.
///
/// Requests an initialized graphical framebuffer from the bootloader, including
/// display dimensions, stride, and color format for early visual output.
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Boot modules request.
///
/// Requests loaded external boot files or modules, such as the initial ramdisk (initramfs).
pub static MODULES_REQUEST: ModulesRequest = ModulesRequest::new();

/// Kernel command line request.
///
/// Requests the boot command-line string passed to the kernel executable by the bootloader.
pub static CMDLINE_REQUEST: ExecutableCmdlineRequest = ExecutableCmdlineRequest::new();
