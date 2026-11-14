//! Memory management subsystems.
//!
//! This module provides abstractions and allocators for managing physical and virtual
//! memory in the kernel, including physical address translation, physical frame
//! allocation, kernel heap allocation (buddy system), page table mapping, and stack allocation.

pub mod address;
pub mod allocator;
pub mod layout;
pub mod mapper;
pub mod stack;
