use crate::fs::{OpenFlags, open_file};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, AgentMeta, TaskControlBlock,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;

/// User-provided arguments for `sys_agent_create`.
#[repr(C)]
pub struct AgentCreateArgs {
    /// Null-terminated executable path in the root filesystem.
    pub path: *const u8,
    /// User-defined agent kind.
    pub agent_type: usize,
    /// Initial heartbeat interval in milliseconds.
    pub heartbeat_interval: usize,
    /// Agent context/resource quota in bytes.
    pub resource_quota: usize,
}

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

/// Create a new Agent process from an executable path.
pub fn sys_agent_create(args_ptr: *const AgentCreateArgs) -> isize {
    let token = current_user_token();
    let args = translated_ref(token, args_ptr);
    let path = translated_str(token, args.path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let parent = current_task().unwrap();
        let all_data = app_inode.read_all();
        let new_task = TaskControlBlock::new_agent(
            &parent,
            all_data.as_slice(),
            args.agent_type,
            args.heartbeat_interval,
            args.resource_quota,
        );
        let pid = new_task.pid.0;
        add_task(new_task);
        pid as isize
    } else {
        -1
    }
}

/// Return Agent metadata for the current process or a direct child.
pub fn sys_agent_info(pid: isize, info_ptr: *mut AgentInfo) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();
    if pid == -1 || pid as usize == task.pid.0 {
        if let Some(meta) = inner.agent {
            fill_agent_info(task.pid.0, meta, info_ptr, token);
            return 0;
        }
        return -2;
    }
    if let Some(child) = inner.children.iter().find(|child| child.pid.0 == pid as usize) {
        let child_inner = child.inner_exclusive_access();
        if let Some(meta) = child_inner.agent {
            fill_agent_info(child.pid.0, meta, info_ptr, token);
            return 0;
        }
        return -2;
    }
    -1
}
