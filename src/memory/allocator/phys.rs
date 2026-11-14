//! # Physical Frame Allocator
//!
//! Provides a bitmap-based physical memory frame allocator ([`PhysFrameAllocator`])
//! that manages 4 KiB physical memory frames reported by the bootloader memory map.
//!
//! The allocator tracks the allocation state of each 4 KiB physical frame using
//! a bitmap, where each bit represents a frame:
//! - `0`: Free / usable frame.
//! - `1`: Reserved, allocated, or unusable frame.
//!
//! It implements both [`FrameAllocator`] and [`FrameDeallocator`] from the `x86_64`
//! crate to integrate with virtual memory paging subsystems.

use limine::memmap::MEMMAP_USABLE;
use spin::{lazylock::LazyLock, mutex::Mutex};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
};

use crate::{boot::info::ENTRIES, memory::address::PhysExt};

pub static PHYS_ALLOCATOR: LazyLock<Mutex<BitmapAllocator>> = LazyLock::new(|| {
    let entries = *ENTRIES;

    let (bitmap_addr, bitmap_base, bitmap_length) = {
        let entry = entries
            .iter()
            .find(|e| e.type_ == MEMMAP_USABLE)
            .expect("No usable memory region found in memory map");

        (
            PhysAddr::new(entry.base).to_virt(),
            entry.base,
            entry.length,
        )
    };

    let highest_addr = entries
        .iter()
        .map(|e| e.base + e.length)
        .max()
        .expect("Memory map is empty");
    let total_frames = (highest_addr).div_ceil(Size4KiB::SIZE) as usize;
    let bitmap_bytes = total_frames.div_ceil(64);

    // SAFETY: The bitmap address is located within a validated usable memory region
    // reported by the bootloader and translated via the Higher-Half Direct Mapping (HHDM).
    // It is exclusively assigned to this allocator during kernel boot.
    let bitmap: &'static mut [u64] = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(bitmap_addr.as_mut_ptr(), bitmap_bytes)
    };

    bitmap.fill(u64::MAX);

    for entry in entries {
        if entry.type_ != MEMMAP_USABLE {
            continue;
        }

        let start_frame = (entry.base / Size4KiB::SIZE) as usize;
        let end_frame = ((entry.base + entry.length) / Size4KiB::SIZE) as usize;

        for frame in start_frame..end_frame {
            let byte = frame / 64;
            let bit = frame % 64;

            bitmap[byte] &= !(1 << bit);
        }
    }

    let start_frame = (bitmap_base / Size4KiB::SIZE) as usize;
    let end_frame = ((bitmap_base + bitmap_length) / Size4KiB::SIZE) as usize;

    for frame in start_frame..end_frame {
        let byte = frame / 64;
        let bit = frame % 64;

        bitmap[byte] |= 1 << bit;
    }

    crate::info!(
        "Physical frame allocator initialized: {} frames tracked (bitmap: {} bytes at {:#x})",
        total_frames,
        bitmap_bytes,
        bitmap_base,
    );

    Mutex::new(BitmapAllocator { bitmap })
});

/// A bitmap-based 4 KiB physical frame allocator.
///
/// Tracks the availability of physical memory frames up to the highest physical
/// memory address discovered in the bootloader's memory map.
///
/// A single bit in the bitmap corresponds to a single 4 KiB physical frame:
/// a cleared bit (`0`) indicates a free frame, while a set bit (`1`) indicates
/// that the frame is allocated, reserved, or invalid.
#[derive(Debug)]
pub struct BitmapAllocator {
    /// Contiguous byte slice backing the frame allocation bitmap.
    bitmap: &'static mut [u64],
}

// SAFETY: `allocate_frame` only returns frames that are within valid usable memory
// regions, marked as free (0) in the bitmap, and marks them as allocated (1) before
// returning, guaranteeing that duplicate or invalid frames are never allocated.
unsafe impl FrameAllocator<Size4KiB> for BitmapAllocator {
    /// Allocates the first available 4 KiB physical frame.
    ///
    /// Searches the bitmap for the first cleared bit (`0`), sets it to mark it
    /// allocated (`1`), and returns the corresponding [`PhysFrame`]. Returns [`None`]
    /// if no free frames remain.
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for (idx, byte) in self.bitmap.iter_mut().enumerate() {
            if *byte == u64::MAX {
                continue;
            }

            let bit = (!*byte).trailing_zeros() as usize;

            *byte |= 1 << bit;

            let frame = idx * 64 + bit;

            return Some(PhysFrame::containing_address(PhysAddr::new(
                (frame * Size4KiB::SIZE as usize) as u64,
            )));
        }

        None
    }
}

impl FrameDeallocator<Size4KiB> for BitmapAllocator {
    /// Deallocates a previously allocated 4 KiB physical frame.
    ///
    /// Clears the corresponding bit in the bitmap to mark the frame as free.
    ///
    /// # Safety
    /// The caller must ensure that the given `frame` was previously allocated by this
    /// allocator and is no longer referenced anywhere in the system.
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame = frame.start_address().as_usize() / Size4KiB::SIZE as usize;

        let byte = frame / 64;
        let bit = frame % 64;

        self.bitmap[byte] &= !(1 << bit);
    }
}
