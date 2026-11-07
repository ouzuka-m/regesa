use crate::error::Result;

pub trait File {
    fn read(&self, buffer: &mut [u8]) -> Result<usize>;

    fn name(&self) -> &str;

    #[allow(dead_code)]
    fn size(&self) -> u64;
}
