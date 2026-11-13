//! Virtual file system and early ramdisk (initramfs) implementation.
//!
//! This module provides file abstractions, filesystem traits, TAR archive parsing,
//! and the early in-memory root filesystem (initramfs) loaded during boot.

pub mod file;
pub mod filesystem;
pub mod initramfs;
pub mod tar;
