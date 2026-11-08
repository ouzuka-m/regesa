use x86_64::instructions::interrupts;

pub fn idle() -> ! {
    loop {
        interrupts::enable_and_hlt();
    }
}
