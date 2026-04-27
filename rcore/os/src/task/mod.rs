//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of [`PidAllocator`] called `PID_ALLOCATOR` allocates
//! pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod manager;
mod pid;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use crate::fs::{OpenFlags, open_file};
use crate::sbi::shutdown;
use crate::timer::get_time_ms;
use alloc::sync::Arc;
use alloc::vec::Vec;
pub use context::TaskContext;
use lazy_static::*;
pub use manager::{TaskManager, fetch_task};
use switch::__switch;
pub(crate) use task::TaskStatus;

pub use manager::add_task;
pub use pid::{KernelStack, PidAllocator, PidHandle, pid_alloc};
pub use processor::{
    Processor, current_task, current_trap_cx, current_user_token, run_tasks, schedule,
    take_current_task,
};
pub(crate) use task::{
    AgentLoopState, AgentMeta, CONTEXT_MAX_NODES, ContextNode, TaskControlBlock,
    TaskControlBlockInner,
};

/// Wake reason bit for heartbeat-triggered Agent Loop iterations.
pub const AGENT_WAKE_HEARTBEAT: usize = 1;
/// Wake reason bit for message-triggered Agent Loop iterations.
pub const AGENT_WAKE_MESSAGE: usize = 2;

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// Block the current task until another kernel event wakes it.
pub fn block_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Blocked;
    drop(task_inner);
    schedule(task_cx_ptr);
}

fn wake_agent_task(task: Arc<TaskControlBlock>, reason: usize) {
    let mut should_enqueue = false;
    {
        let mut inner = task.inner_exclusive_access();
        if let Some(meta) = inner.agent.as_mut() {
            meta.pending_wake_reason |= reason;
            meta.loop_state = AgentLoopState::Ready;
            if inner.task_status == TaskStatus::Blocked {
                inner.task_status = TaskStatus::Ready;
                should_enqueue = true;
            }
        }
    }
    if should_enqueue {
        add_task(task);
    }
}

fn collect_children(task: &Arc<TaskControlBlock>) -> Vec<Arc<TaskControlBlock>> {
    let inner = task.inner_exclusive_access();
    let children = inner.children.clone();
    drop(inner);
    children
}

fn check_agent_heartbeats_from(task: Arc<TaskControlBlock>, now_ms: usize) {
    let mut should_wake = false;
    {
        let mut inner = task.inner_exclusive_access();
        if let Some(meta) = inner.agent.as_mut() {
            if meta.heartbeat_interval > 0
                && meta.heartbeat_next_at > 0
                && now_ms >= meta.heartbeat_next_at
            {
                meta.heartbeat_next_at = now_ms + meta.heartbeat_interval;
                meta.pending_wake_reason |= AGENT_WAKE_HEARTBEAT;
                meta.loop_state = AgentLoopState::Ready;
                if inner.task_status == TaskStatus::Blocked {
                    inner.task_status = TaskStatus::Ready;
                    should_wake = true;
                }
            }
        }
    }
    if should_wake {
        add_task(task.clone());
    }
    for child in collect_children(&task) {
        check_agent_heartbeats_from(child, now_ms);
    }
}

/// Wake blocked Agents whose heartbeat deadline has arrived.
pub fn check_agent_heartbeats() {
    check_agent_heartbeats_from(INITPROC.clone(), get_time_ms());
}

/// Wake an Agent by pid and record a semantic wake reason.
pub fn wake_agent_by_pid(pid: usize, reason: usize) -> bool {
    fn walk(task: Arc<TaskControlBlock>, pid: usize, reason: usize) -> bool {
        if task.pid.0 == pid {
            wake_agent_task(task, reason);
            return true;
        }
        for child in collect_children(&task) {
            if walk(child, pid, reason) {
                return true;
            }
        }
        false
    }
    walk(INITPROC.clone(), pid, reason)
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}
