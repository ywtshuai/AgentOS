#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use buddy_system_allocator::LockedHeap;
use core::ptr::addr_of_mut;
use syscall::*;
pub use syscall::{
    AgentInfo, CONTEXT_QUERY_MAX_NODES, ContextNode, ContextPushRequest, ContextQueryRequest,
    ContextQueryResult, TOOL_GET_SYSTEM_STATUS, TOOL_MAX_PARAMS, TOOL_PARAM_AGENT_TYPE,
    TOOL_PARAM_STATUS, TOOL_PARAM_TARGET_PID, TOOL_QUERY_MAX_ITEMS, TOOL_QUERY_PROCESS,
    TOOL_SEND_MESSAGE, TOOL_VALUE_U64, ToolInfo, ToolMessageResult, ToolParam,
    ToolProcessQueryResult, ToolProcessSummary, ToolRequest, ToolResponse, ToolSystemStatus,
};

const USER_HEAP_SIZE: usize = 32768;
pub const AGENT_CONTEXT_SIZE: usize = 64 * 1024;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}
pub fn agent_create(agent_type: usize, heartbeat_interval: usize, resource_quota: usize) -> isize {
    sys_agent_create(agent_type, heartbeat_interval, resource_quota)
}
pub fn agent_info(pid: isize, info: &mut AgentInfo) -> isize {
    sys_agent_info(pid, info)
}
pub fn tool_call(request: &ToolRequest, response: &mut ToolResponse) -> isize {
    sys_tool_call(request, response)
}
pub fn tool_list(info: &mut [ToolInfo]) -> isize {
    sys_tool_list(info)
}
pub fn context_push(request: &ContextPushRequest, node: &mut ContextNode) -> isize {
    sys_context_push(request, node)
}
pub fn context_query(request: &ContextQueryRequest, result: &mut ContextQueryResult) -> isize {
    sys_context_query(request, result)
}
pub fn context_rollback(node_id: usize) -> isize {
    sys_context_rollback(node_id)
}
pub fn context_clear() -> isize {
    sys_context_clear()
}
