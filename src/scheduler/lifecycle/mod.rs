//! Task lifecycle routines for the scheduler.
//!
//! This module manages task state transitions and core lifecycle loops,
//! providing the exit path for terminated tasks and the idle task loop.

pub mod exit;
pub mod idle;
