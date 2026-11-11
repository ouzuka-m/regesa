use x86_64::instructions::interrupts;

pub fn exit() -> ! {
    loop {
        interrupts::enable_and_hlt();
    }
}
