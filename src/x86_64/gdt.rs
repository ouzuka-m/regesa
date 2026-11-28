//! # Global Descriptor Table (GDT) and Segment Configuration
//!
//! Configures the 64-bit x86_64 Global Descriptor Table (GDT) and loads segment registers
//! for the kernel environment. Although segmentation is largely disabled in 64-bit mode,
//! valid code and data segment descriptors as well as a Task State Segment (TSS) descriptor
//! must still be present in the GDT.

use spin::lazylock::LazyLock;
use x86_64::{
    instructions::tables,
    registers::segmentation::{CS, SS, Segment},
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
};

use crate::x86_64::tss::TASK_STATE_SEGMENT;

/// Holds the segment selectors generated during GDT setup.
pub struct Selectors {
    /// Segment selector for the Task State Segment (TSS).
    tss: SegmentSelector,
    /// Segment selector for the 64-bit kernel code segment.
    pub(crate) kernel_code: SegmentSelector,
    /// Segment selector for the 64-bit kernel data segment.
    pub(crate) kernel_data: SegmentSelector,
    /// Segment selector for the 64-bit user data segment.
    pub(crate) user_data: SegmentSelector,
    /// Segment selector for the 64-bit user code segment.
    pub(crate) user_code: SegmentSelector,
}

/// Static lazy initialization of the GDT and its associated segment selectors.
static GDT_STATE: LazyLock<(GlobalDescriptorTable, Selectors)> = LazyLock::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let tss = gdt.append(Descriptor::tss_segment(&TASK_STATE_SEGMENT));

    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());

    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());

    let selectors = Selectors {
        tss,
        kernel_code,
        kernel_data,
        user_data,
        user_code,
    };

    (gdt, selectors)
});

/// Initializes and loads the Global Descriptor Table (GDT) and Task State Segment (TSS).
///
/// Loads the newly constructed GDT into the CPU using `lgdt`, updates the Code Segment (`CS`)
/// and Stack Segment (`SS`) registers to point to the new kernel code and data selectors,
/// and executes `ltr` to load the Task State Segment selector.
///
/// # Safety
/// Reloading segment registers relies on valid segment selectors configured in the GDT.
pub fn init() {
    let (gdt, selectors) = &*GDT_STATE;

    // Load GDT pointer into CPU descriptor register (GDTR)
    gdt.load();
    // SAFETY: The segment selectors were just created from valid GDT
    // descriptors that we built and loaded above. Reloading CS, SS,
    // and TR with these selectors is required after loading a new GDT.
    unsafe {
        CS::set_reg(selectors.kernel_code);
        SS::set_reg(selectors.kernel_data);
        tables::load_tss(selectors.tss);
    }

    crate::info!("GDT and TSS loaded");
}

pub fn selectors() -> &'static Selectors {
    &GDT_STATE.1
}
