#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn entry() {
    core::arch::naked_asm!(
        "
        sysretq
        "
    )
}
