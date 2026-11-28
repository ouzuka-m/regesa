use x86_64::registers::model_specific::Msr;

#[allow(dead_code)]
pub fn rdmsr(msr: u32) -> u64 {
    unsafe { Msr::new(msr).read() }
}

pub fn wrmsr(msr: u32, value: u64) {
    unsafe { Msr::new(msr).write(value) }
}
