//! Cooperative and preemptive task scheduler.
//!
//! This module provides the task scheduling infrastructure for the kernel,
//! implementing a fixed-capacity round-robin scheduler (`RoundRobin<N_TASKS>`)
//! along with low-level x86_64 context switching mechanisms.
//!
//! # Architecture & Context Switching
//!
//! Task execution contexts are defined by [`context::Context`], which captures the
//! callee-saved registers (`r15`–`r12`, `rbx`, `rbp`), the stack pointer (`rsp`),
//! and the page table root register (`cr3`) for per-task virtual address spaces.
//!
//! State transitions are dispatched through [`Switch`]:
//! - [`Switch::Launch`]: Boots the initial task into execution via [`context::launch`].
//! - [`Switch::Swap`]: Saves the active task's register state and flags onto its stack
//!   and context structure, restores the incoming task's registers, conditionally reloads `cr3`
//!   if address spaces differ, and resumes execution via [`context::switch`].
//! - [`Switch::None`]: Indicates that no context transition is required.
//!
//! Scheduling is globally gated by an atomic flag, enabled after bootstrap via [`activate()`].

pub mod context;
pub mod lifecycle;
pub mod rflags;
pub mod round_robin;
pub mod state;
pub mod switch;
pub mod task;

use core::sync::atomic::{AtomicBool, Ordering};
use spin::{lazylock::LazyLock, mutex::Mutex};

use self::{round_robin::RoundRobin, switch::Switch};

const N_TASKS: usize = 16;

pub static SCHEDULER: LazyLock<Mutex<RoundRobin<N_TASKS>>> =
    LazyLock::new(|| Mutex::new(RoundRobin::new()));

static SCHEDULER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Activates the scheduler, allowing calls to [`schedule`] to perform task switching.
pub fn activate() {
    SCHEDULER_ACTIVE.store(true, Ordering::Release);
}

/// Triggers a scheduling pass.
///
/// If scheduling is enabled via [`activate`], queries the underlying scheduler policy
/// for the next [`Switch`] action and executes the corresponding context transition
/// via [`context::launch`] or [`context::switch`].
pub fn schedule() {
    if !SCHEDULER_ACTIVE.load(Ordering::Acquire) {
        return;
    }

    let switch = SCHEDULER.lock().prepare();

    unsafe {
        match switch {
            Switch::Launch(ctx) => context::launch(ctx),
            Switch::Swap(old, new) => context::switch(old, new),
            _ => {}
        }
    }
}

/// Initializes the global task scheduler instance.
pub fn init() {
    LazyLock::force(&SCHEDULER);
}
