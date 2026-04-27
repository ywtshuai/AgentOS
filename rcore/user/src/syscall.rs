use core::arch::asm;

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

pub const TOOL_GET_SYSTEM_STATUS: usize = 1;
pub const TOOL_QUERY_PROCESS: usize = 2;
pub const TOOL_SEND_MESSAGE: usize = 3;
pub const TOOL_MAX_PARAMS: usize = 4;
pub const TOOL_QUERY_MAX_ITEMS: usize = 8;
pub const TOOL_PARAM_STATUS: usize = 1;
pub const TOOL_PARAM_AGENT_TYPE: usize = 2;
pub const TOOL_PARAM_TARGET_PID: usize = 10;
pub const TOOL_VALUE_U64: usize = 1;
pub const CONTEXT_QUERY_MAX_NODES: usize = 8;
pub const AGENT_WAKE_HEARTBEAT: usize = 1;
pub const AGENT_WAKE_MESSAGE: usize = 2;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct AgentInfo {
    pub pid: usize,
    pub agent_type: usize,
    pub heartbeat_interval: usize,
    pub heartbeat_next_at: usize,
    pub pending_wake_reason: usize,
    pub pending_messages: usize,
    pub resource_quota: usize,
    pub loop_state: usize,
    pub context_path_meta: usize,
    pub agent_context_base: usize,
    pub agent_context_size: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolParam {
    pub key_id: usize,
    pub value_type: usize,
    pub value: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolRequest {
    pub tool_id: usize,
    pub param_count: usize,
    pub params: [ToolParam; TOOL_MAX_PARAMS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolResponse {
    pub status: isize,
    pub result_len: usize,
    pub result_offset: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolInfo {
    pub tool_id: usize,
    pub max_params: usize,
    pub flags: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolSystemStatus {
    pub process_count: usize,
    pub agent_count: usize,
    pub ready_count: usize,
    pub running_count: usize,
    pub zombie_count: usize,
    pub current_pid: usize,
    pub time_ms: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolProcessSummary {
    pub pid: usize,
    pub status: usize,
    pub agent_type: usize,
    pub loop_state: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolProcessQueryResult {
    pub total_matches: usize,
    pub returned: usize,
    pub items: [ToolProcessSummary; TOOL_QUERY_MAX_ITEMS],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ToolMessageResult {
    pub target_pid: usize,
    pub target_agent_type: usize,
    pub accepted: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ContextNode {
    pub node_id: usize,
    pub prev_id: usize,
    pub timestamp: usize,
    pub tool_id: usize,
    pub request_offset: usize,
    pub request_len: usize,
    pub result_offset: usize,
    pub result_len: usize,
    pub node_offset: usize,
    pub flags: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ContextPushRequest {
    pub tool_id: usize,
    pub flags: usize,
    pub request_ptr: usize,
    pub request_len: usize,
    pub result_ptr: usize,
    pub result_len: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ContextQueryRequest {
    pub start_index: usize,
    pub max_nodes: usize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ContextQueryResult {
    pub total_nodes: usize,
    pub returned: usize,
    pub active_node_id: usize,
    pub write_offset: usize,
    pub nodes: [ContextNode; CONTEXT_QUERY_MAX_NODES],
}

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn sys_agent_create(
    agent_type: usize,
    heartbeat_interval: usize,
    resource_quota: usize,
) -> isize {
    syscall(
        SYSCALL_AGENT_CREATE,
        [agent_type, heartbeat_interval, resource_quota],
    )
}

pub fn sys_agent_info(pid: isize, info: &mut AgentInfo) -> isize {
    syscall(
        SYSCALL_AGENT_INFO,
        [pid as usize, info as *mut _ as usize, 0],
    )
}

pub fn sys_tool_call(request: &ToolRequest, response: &mut ToolResponse) -> isize {
    syscall(
        SYSCALL_TOOL_CALL,
        [request as *const _ as usize, response as *mut _ as usize, 0],
    )
}

pub fn sys_tool_list(info: &mut [ToolInfo]) -> isize {
    syscall(
        SYSCALL_TOOL_LIST,
        [info.as_mut_ptr() as usize, info.len(), 0],
    )
}

pub fn sys_context_push(request: &ContextPushRequest, node: &mut ContextNode) -> isize {
    syscall(
        SYSCALL_CONTEXT_PUSH,
        [request as *const _ as usize, node as *mut _ as usize, 0],
    )
}

pub fn sys_context_query(request: &ContextQueryRequest, result: &mut ContextQueryResult) -> isize {
    syscall(
        SYSCALL_CONTEXT_QUERY,
        [request as *const _ as usize, result as *mut _ as usize, 0],
    )
}

pub fn sys_context_rollback(node_id: usize) -> isize {
    syscall(SYSCALL_CONTEXT_ROLLBACK, [node_id, 0, 0])
}

pub fn sys_context_clear() -> isize {
    syscall(SYSCALL_CONTEXT_CLEAR, [0, 0, 0])
}

pub fn sys_agent_heartbeat_set(interval_ms: usize) -> isize {
    syscall(SYSCALL_AGENT_HEARTBEAT_SET, [interval_ms, 0, 0])
}

pub fn sys_agent_heartbeat_stop() -> isize {
    syscall(SYSCALL_AGENT_HEARTBEAT_STOP, [0, 0, 0])
}

pub fn sys_agent_wait() -> isize {
    syscall(SYSCALL_AGENT_WAIT, [0, 0, 0])
}
