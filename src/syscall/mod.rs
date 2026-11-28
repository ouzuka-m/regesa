pub mod dispatch;
pub mod entry;
pub mod handlers;

use x86_64::{
    VirtAddr,
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
};

use crate::x86_64::gdt;

pub fn init() {
    unsafe {
        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }

    let selectors = gdt::selectors();
    Star::write(
        selectors.user_code,
        selectors.user_data,
        selectors.kernel_code,
        selectors.kernel_data,
    )
    .expect("Invalid CS/SS value for SYSCALL/SYSRET");

    LStar::write(VirtAddr::new(entry::entry as *const () as u64));

    SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::DIRECTION_FLAG);
}
