use x86_64::instructions::interrupts;

use crate::scheduler::SCHEDULER;

pub fn exit() -> ! {
    interrupts::without_interrupts(|| {
        SCHEDULER.lock().release_current();
    });

    loop {
        interrupts::enable_and_hlt();
    }
}
