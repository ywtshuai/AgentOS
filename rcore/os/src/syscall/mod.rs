//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_AGENT_CREATE: usize = 500;
const SYSCALL_AGENT_INFO: usize = 501;
const SYSCALL_TOOL_CALL: usize = 502;
const SYSCALL_TOOL_LIST: usize = 503;
const SYSCALL_CONTEXT_PUSH: usize = 504;
const SYSCALL_CONTEXT_QUERY: usize = 505;
const SYSCALL_CONTEXT_ROLLBACK: usize = 506;
const SYSCALL_CONTEXT_CLEAR: usize = 507;
const SYSCALL_AGENT_HEARTBEAT_SET: usize = 508;
const SYSCALL_AGENT_HEARTBEAT_STOP: usize = 509;
const SYSCALL_AGENT_WAIT: usize = 510;

mod fs;
mod process;

use crate::task::ContextNode;
use fs::*;
use process::*;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_AGENT_CREATE => sys_agent_create(args[0], args[1], args[2]),
        SYSCALL_AGENT_INFO => sys_agent_info(args[0] as isize, args[1] as *mut AgentInfo),
        SYSCALL_TOOL_CALL => {
            sys_tool_call(args[0] as *const ToolRequest, args[1] as *mut ToolResponse)
        }
        SYSCALL_TOOL_LIST => sys_tool_list(args[0] as *mut ToolInfo, args[1]),
        SYSCALL_CONTEXT_PUSH => sys_context_push(
            args[0] as *const ContextPushRequest,
            args[1] as *mut ContextNode,
        ),
        SYSCALL_CONTEXT_QUERY => sys_context_query(
            args[0] as *const ContextQueryRequest,
            args[1] as *mut ContextQueryResult,
        ),
        SYSCALL_CONTEXT_ROLLBACK => sys_context_rollback(args[0]),
        SYSCALL_CONTEXT_CLEAR => sys_context_clear(),
        SYSCALL_AGENT_HEARTBEAT_SET => sys_agent_heartbeat_set(args[0]),
        SYSCALL_AGENT_HEARTBEAT_STOP => sys_agent_heartbeat_stop(),
        SYSCALL_AGENT_WAIT => sys_agent_wait(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
