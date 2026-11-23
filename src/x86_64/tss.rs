//! # Task State Segment (TSS) and Interrupt Stacks
//!
//! Configures the x86_64 Task State Segment (TSS) and sets up the Interrupt Stack Table (IST).
//! Dedicated stacks are allocated for critical exceptions (Double Fault, Non-Maskable Interrupt,
//! and Machine Check Exception) to prevent stack overflow cascading into a hardware triple fault.

use core::ops::Add;

use spin::lazylock::LazyLock;
use x86_64::{VirtAddr, structures::tss::TaskStateSegment};

use crate::x86_64::ist::{self, Stack};

pub static TASK_STATE_SEGMENT: LazyLock<TaskStateSegment> = LazyLock::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.privilege_stack_table[ist::KERNEL_INDEX as usize] = stack_end(&raw const ist::KERNEL_STACK);

    tss.interrupt_stack_table[ist::DF_INDEX as usize] = stack_end(&raw const ist::DF_STACK);
    tss.interrupt_stack_table[ist::NMI_INDEX as usize] = stack_end(&raw const ist::NMI_STACK);
    tss.interrupt_stack_table[ist::MCE_INDEX as usize] = stack_end(&raw const ist::MCE_STACK);

    tss
});

fn stack_end(stack_ptr: *const Stack) -> VirtAddr {
    VirtAddr::from_ptr(stack_ptr).add(ist::STACK_SIZE as u64)
}
