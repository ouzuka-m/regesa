pub mod context;
pub mod exit;
pub mod idle;
pub mod rflags;
pub mod round_robin;
pub mod state;
pub mod switch;
pub mod task;

use core::sync::atomic::{AtomicBool, Ordering};
use spin::{lazylock::LazyLock, mutex::Mutex};

use self::{round_robin::RoundRobin, switch::Switch};

pub static SCHEDULER_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static SCHEDULER: LazyLock<Mutex<RoundRobin<255>>> =
    LazyLock::new(|| Mutex::new(RoundRobin::new()));

pub fn activate() {
    SCHEDULER_ACTIVE.store(true, Ordering::Release);
}

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
