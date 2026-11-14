#![allow(dead_code)]

pub const GAP: usize = 1024 * 1024;

pub const HEAP_SIZE: usize = 16 * (1024 * 1024); // 16 MB, 4096 pages
pub const HEAP_START: usize = 0xFFFF_9000_0000_0000;
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

pub const STACK_SIZE: usize = 4 * (1024 * 1024); // 4 MB, 1024 pages
pub const STACK_BOTTOM: usize = HEAP_END + GAP;
pub const STACK_TOP: usize = STACK_BOTTOM + STACK_SIZE;
