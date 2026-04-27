use crate::config::{AGENT_CONTEXT_BASE, AGENT_CONTEXT_SIZE};
use crate::fs::{OpenFlags, open_file};
use crate::mm::{
    MapPermission, translated_byte_buffer, translated_ref, translated_refmut, translated_str,
};
use crate::task::{
    AgentLoopState, AgentMeta, INITPROC, TaskControlBlock, TaskStatus, add_task, current_task,
    current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;
use core::cmp::min;
use core::mem::size_of;

pub const TOOL_GET_SYSTEM_STATUS: usize = 1;
pub const TOOL_QUERY_PROCESS: usize = 2;
pub const TOOL_SEND_MESSAGE: usize = 3;
const TOOL_MAX_PARAMS: usize = 4;
const TOOL_QUERY_MAX_ITEMS: usize = 8;

const TOOL_STATUS_OK: isize = 0;
const TOOL_ERR_NOT_AGENT: isize = -3;
const TOOL_ERR_UNKNOWN_TOOL: isize = -4;
const TOOL_ERR_BAD_PARAM: isize = -5;
const TOOL_ERR_CONTEXT_FULL: isize = -6;
const TOOL_ERR_NOT_FOUND: isize = -7;

const TOOL_PARAM_STATUS: usize = 1;
const TOOL_PARAM_AGENT_TYPE: usize = 2;
const TOOL_PARAM_TARGET_PID: usize = 10;
const TOOL_VALUE_U64: usize = 1;

/// User-visible Agent metadata returned by `sys_agent_info`.
#[repr(C)]
pub struct AgentInfo {
    /// Process id.
    pub pid: usize,
    /// User-defined agent kind.
    pub agent_type: usize,
    /// Heartbeat interval in milliseconds.
    pub heartbeat_interval: usize,
    /// Context/resource quota in bytes.
    pub resource_quota: usize,
    /// Agent loop state encoded as an integer.
    pub loop_state: usize,
    /// Reserved context-path metadata slot.
    pub context_path_meta: usize,
    /// Base virtual address of Agent Context.
    pub agent_context_base: usize,
    /// Size of Agent Context in bytes.
    pub agent_context_size: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ToolParam {
    pub key_id: usize,
    pub value_type: usize,
    pub value: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ToolRequest {
    pub tool_id: usize,
    pub param_count: usize,
    pub params: [ToolParam; TOOL_MAX_PARAMS],
}

#[repr(C)]
pub struct ToolResponse {
    pub status: isize,
    pub result_len: usize,
    pub result_offset: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ToolInfo {
    pub tool_id: usize,
    pub max_params: usize,
    pub flags: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
pub struct ToolProcessSummary {
    pub pid: usize,
    pub status: usize,
    pub agent_type: usize,
    pub loop_state: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ToolProcessQueryResult {
    pub total_matches: usize,
    pub returned: usize,
    pub items: [ToolProcessSummary; TOOL_QUERY_MAX_ITEMS],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ToolMessageResult {
    pub target_pid: usize,
    pub target_agent_type: usize,
    pub accepted: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// Mark the current process as an Agent and map its Agent Context area.
pub fn sys_agent_create(
    agent_type: usize,
    heartbeat_interval: usize,
    resource_quota: usize,
) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if inner.agent.is_some() {
        return -1;
    }
    if !inner.memory_set.insert_framed_area_checked(
        AGENT_CONTEXT_BASE.into(),
        (AGENT_CONTEXT_BASE + AGENT_CONTEXT_SIZE).into(),
        MapPermission::R | MapPermission::W | MapPermission::U,
    ) {
        return -2;
    }
    inner.agent = Some(AgentMeta::new(
        agent_type,
        heartbeat_interval,
        resource_quota,
    ));
    0
}

fn fill_agent_info(pid: usize, meta: AgentMeta, info_ptr: *mut AgentInfo, token: usize) {
    *translated_refmut(token, info_ptr) = AgentInfo {
        pid,
        agent_type: meta.agent_type,
        heartbeat_interval: meta.heartbeat_interval,
        resource_quota: meta.resource_quota,
        loop_state: meta.loop_state as usize,
        context_path_meta: meta.context_path_meta,
        agent_context_base: meta.agent_context_base,
        agent_context_size: meta.agent_context_size,
    };
}

fn find_agent_meta(task: Arc<TaskControlBlock>, pid: usize) -> Option<(usize, Option<AgentMeta>)> {
    let inner = task.inner_exclusive_access();
    if task.pid.0 == pid {
        return Some((task.pid.0, inner.agent));
    }
    let children = inner.children.clone();
    drop(inner);
    for child in children {
        if let Some(meta) = find_agent_meta(child, pid) {
            return Some(meta);
        }
    }
    None
}

/// Return Agent metadata for the current process or a process in the task tree.
pub fn sys_agent_info(pid: isize, info_ptr: *mut AgentInfo) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let target_pid = if pid == -1 { task.pid.0 } else { pid as usize };
    if let Some((found_pid, meta)) = find_agent_meta(INITPROC.clone(), target_pid) {
        if let Some(meta) = meta {
            fill_agent_info(found_pid, meta, info_ptr, token);
            0
        } else {
            -2
        }
    } else {
        -1
    }
}

fn status_code(status: TaskStatus) -> usize {
    match status {
        TaskStatus::Ready => 0,
        TaskStatus::Running => 1,
        TaskStatus::Zombie => 2,
    }
}

fn result_bytes<T>(result: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(result as *const T as *const u8, size_of::<T>()) }
}

fn write_agent_context_result(
    meta: &mut AgentMeta,
    token: usize,
    bytes: &[u8],
) -> Result<usize, isize> {
    let mut quota = meta.resource_quota;
    if quota == 0 || quota > meta.agent_context_size {
        quota = meta.agent_context_size;
    }
    if bytes.len() > quota {
        return Err(TOOL_ERR_CONTEXT_FULL);
    }
    if meta.context_path_meta + bytes.len() > quota {
        meta.context_path_meta = 0;
    }
    let offset = meta.context_path_meta;
    let dst = (meta.agent_context_base + offset) as *const u8;
    let buffers = translated_byte_buffer(token, dst, bytes.len());
    let mut copied = 0;
    for buffer in buffers {
        let len = buffer.len();
        buffer.copy_from_slice(&bytes[copied..copied + len]);
        copied += len;
    }
    meta.context_path_meta = offset + bytes.len();
    Ok(offset)
}

fn write_tool_response(
    token: usize,
    response_ptr: *mut ToolResponse,
    status: isize,
    result_len: usize,
    result_offset: usize,
) -> isize {
    *translated_refmut(token, response_ptr) = ToolResponse {
        status,
        result_len,
        result_offset,
    };
    status
}

fn walk_tasks<F: FnMut(&TaskControlBlock, &crate::task::TaskControlBlockInner)>(
    task: Arc<TaskControlBlock>,
    f: &mut F,
) {
    let inner = task.inner_exclusive_access();
    f(&task, &inner);
    let children = inner.children.clone();
    drop(inner);
    for child in children {
        walk_tasks(child, f);
    }
}

fn find_agent_task_meta(task: Arc<TaskControlBlock>, pid: usize) -> Option<(usize, AgentMeta)> {
    let inner = task.inner_exclusive_access();
    if task.pid.0 == pid {
        return inner.agent.map(|meta| (task.pid.0, meta));
    }
    let children = inner.children.clone();
    drop(inner);
    for child in children {
        if let Some(meta) = find_agent_task_meta(child, pid) {
            return Some(meta);
        }
    }
    None
}

fn get_param(request: &ToolRequest, key_id: usize) -> Option<usize> {
    if request.param_count > TOOL_MAX_PARAMS {
        return None;
    }
    for i in 0..request.param_count {
        let param = request.params[i];
        if param.key_id == key_id {
            if param.value_type != TOOL_VALUE_U64 {
                return None;
            }
            return Some(param.value);
        }
    }
    None
}

fn validate_u64_params(request: &ToolRequest) -> bool {
    if request.param_count > TOOL_MAX_PARAMS {
        return false;
    }
    for i in 0..request.param_count {
        if request.params[i].value_type != TOOL_VALUE_U64 {
            return false;
        }
    }
    true
}

fn build_system_status() -> ToolSystemStatus {
    let mut result = ToolSystemStatus {
        process_count: 0,
        agent_count: 0,
        ready_count: 0,
        running_count: 0,
        zombie_count: 0,
        current_pid: current_task().unwrap().pid.0,
        time_ms: get_time_ms(),
    };
    walk_tasks(INITPROC.clone(), &mut |_, inner| {
        result.process_count += 1;
        if inner.agent.is_some() {
            result.agent_count += 1;
        }
        match inner.task_status {
            TaskStatus::Ready => result.ready_count += 1,
            TaskStatus::Running => result.running_count += 1,
            TaskStatus::Zombie => result.zombie_count += 1,
        }
    });
    result
}

fn build_process_query(request: &ToolRequest) -> Result<ToolProcessQueryResult, isize> {
    if !validate_u64_params(request) {
        return Err(TOOL_ERR_BAD_PARAM);
    }
    let status_filter = get_param(request, TOOL_PARAM_STATUS);
    let agent_type_filter = get_param(request, TOOL_PARAM_AGENT_TYPE);
    let empty = ToolProcessSummary {
        pid: 0,
        status: 0,
        agent_type: 0,
        loop_state: 0,
    };
    let mut result = ToolProcessQueryResult {
        total_matches: 0,
        returned: 0,
        items: [empty; TOOL_QUERY_MAX_ITEMS],
    };
    walk_tasks(INITPROC.clone(), &mut |task, inner| {
        let status = status_code(inner.task_status);
        if let Some(filter) = status_filter {
            if status != filter {
                return;
            }
        }
        let agent_type = inner.agent.map(|meta| meta.agent_type).unwrap_or(0);
        if let Some(filter) = agent_type_filter {
            if agent_type != filter {
                return;
            }
        }
        result.total_matches += 1;
        if result.returned < TOOL_QUERY_MAX_ITEMS {
            result.items[result.returned] = ToolProcessSummary {
                pid: task.pid.0,
                status,
                agent_type,
                loop_state: inner
                    .agent
                    .map(|meta| meta.loop_state as usize)
                    .unwrap_or(AgentLoopState::Ready as usize),
            };
            result.returned += 1;
        }
    });
    Ok(result)
}

pub fn sys_tool_call(request_ptr: *const ToolRequest, response_ptr: *mut ToolResponse) -> isize {
    let token = current_user_token();
    let request = *translated_ref(token, request_ptr);
    if request.param_count > TOOL_MAX_PARAMS {
        return write_tool_response(token, response_ptr, TOOL_ERR_BAD_PARAM, 0, 0);
    }
    let task = current_task().unwrap();
    if task.inner_exclusive_access().agent.is_none() {
        return write_tool_response(token, response_ptr, TOOL_ERR_NOT_AGENT, 0, 0);
    }

    let write_result = |bytes: &[u8]| -> isize {
        let mut inner = task.inner_exclusive_access();
        let meta = inner.agent.as_mut().unwrap();
        match write_agent_context_result(meta, token, bytes) {
            Ok(offset) => {
                write_tool_response(token, response_ptr, TOOL_STATUS_OK, bytes.len(), offset)
            }
            Err(err) => write_tool_response(token, response_ptr, err, 0, 0),
        }
    };

    match request.tool_id {
        TOOL_GET_SYSTEM_STATUS => {
            if request.param_count != 0 {
                return write_tool_response(token, response_ptr, TOOL_ERR_BAD_PARAM, 0, 0);
            }
            let result = build_system_status();
            write_result(result_bytes(&result))
        }
        TOOL_QUERY_PROCESS => match build_process_query(&request) {
            Ok(result) => write_result(result_bytes(&result)),
            Err(err) => write_tool_response(token, response_ptr, err, 0, 0),
        },
        TOOL_SEND_MESSAGE => {
            if request.param_count != 1 {
                return write_tool_response(token, response_ptr, TOOL_ERR_BAD_PARAM, 0, 0);
            }
            let target_pid = if let Some(pid) = get_param(&request, TOOL_PARAM_TARGET_PID) {
                pid
            } else {
                return write_tool_response(token, response_ptr, TOOL_ERR_BAD_PARAM, 0, 0);
            };
            if let Some((pid, target_meta)) = find_agent_task_meta(INITPROC.clone(), target_pid) {
                let result = ToolMessageResult {
                    target_pid: pid,
                    target_agent_type: target_meta.agent_type,
                    accepted: 1,
                };
                write_result(result_bytes(&result))
            } else {
                write_tool_response(token, response_ptr, TOOL_ERR_NOT_FOUND, 0, 0)
            }
        }
        _ => write_tool_response(token, response_ptr, TOOL_ERR_UNKNOWN_TOOL, 0, 0),
    }
}

pub fn sys_tool_list(info_ptr: *mut ToolInfo, capacity: usize) -> isize {
    let token = current_user_token();
    let tools = [
        ToolInfo {
            tool_id: TOOL_GET_SYSTEM_STATUS,
            max_params: 0,
            flags: 0,
        },
        ToolInfo {
            tool_id: TOOL_QUERY_PROCESS,
            max_params: TOOL_MAX_PARAMS,
            flags: 0,
        },
        ToolInfo {
            tool_id: TOOL_SEND_MESSAGE,
            max_params: 1,
            flags: 0,
        },
    ];
    let n = min(capacity, tools.len());
    for i in 0..n {
        *translated_refmut(token, unsafe { info_ptr.add(i) }) = tools[i];
    }
    tools.len() as isize
}
