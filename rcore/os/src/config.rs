//! Constants used in rCore
#[allow(unused)]

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const AGENT_CONTEXT_SIZE: usize = 64 * 1024;
pub const AGENT_CONTEXT_BASE: usize = TRAP_CONTEXT - AGENT_CONTEXT_SIZE;

pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};
