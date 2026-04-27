use crate::fs::{OpenFlags, open_file};
use crate::mm::{translated_refmut, translated_str};
use crate::task::{
    AgentMeta, INITPROC, TaskControlBlock, add_task, current_task, current_user_token,
    exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;

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
