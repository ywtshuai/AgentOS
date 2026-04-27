//!Implementation of [`TaskControlBlock`]
use super::TaskContext;
use super::{KernelStack, PidHandle, pid_alloc};
use crate::config::{AGENT_CONTEXT_BASE, AGENT_CONTEXT_SIZE, TRAP_CONTEXT};
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{KERNEL_SPACE, MapPermission, MemorySet, PhysPageNum, VirtAddr};
use crate::sync::UPSafeCell;
use crate::trap::{TrapContext, trap_handler};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    #[allow(unused)]
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    /// Agent-specific metadata. `None` means this is a regular process.
    pub agent: Option<AgentMeta>,
}

/// Kernel-side metadata tracked for an Agent process.
#[derive(Copy, Clone)]
pub struct AgentMeta {
    /// User-defined agent kind.
    pub agent_type: usize,
    /// Requested heartbeat period in milliseconds.
    pub heartbeat_interval: usize,
    /// Next heartbeat deadline in milliseconds. Zero means no heartbeat is armed.
    pub heartbeat_next_at: usize,
    /// Pending wake reason bits consumed by `sys_agent_wait`.
    pub pending_wake_reason: usize,
    /// Count of pending structured messages.
    pub pending_messages: usize,
    /// Context/resource quota in bytes.
    pub resource_quota: usize,
    /// Agent loop state used by later milestones.
    pub loop_state: AgentLoopState,
    /// Reserved context-path metadata slot.
    pub context_path_meta: usize,
    /// Number of live context nodes tracked by the kernel.
    pub context_node_count: usize,
    /// Next Agent Context byte offset used by tool results and context nodes.
    pub context_write_offset: usize,
    /// Current active context node id. Zero means there is no active node.
    pub context_active_node: usize,
    /// Next context node id to allocate.
    pub context_next_node: usize,
    /// FIFO metadata ring for recent Context Path nodes.
    pub context_nodes: [ContextNode; CONTEXT_MAX_NODES],
    /// Base virtual address of the mapped Agent Context area, initialized in M2.
    pub agent_context_base: usize,
    /// Size of the mapped Agent Context area, initialized in M2.
    pub agent_context_size: usize,
}

pub const CONTEXT_MAX_NODES: usize = 16;

#[repr(C)]
#[derive(Copy, Clone)]
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

impl ContextNode {
    pub const fn empty() -> Self {
        Self {
            node_id: 0,
            prev_id: 0,
            timestamp: 0,
            tool_id: 0,
            request_offset: 0,
            request_len: 0,
            result_offset: 0,
            result_len: 0,
            node_offset: 0,
            flags: 0,
        }
    }
}

/// Minimal Agent loop state for M1 metadata.
#[repr(usize)]
#[allow(unused)]
#[derive(Copy, Clone, PartialEq)]
pub enum AgentLoopState {
    /// Ready to run.
    Ready = 0,
    /// Waiting for an event.
    Waiting = 1,
    /// Currently running.
    Running = 2,
    /// Finished execution.
    Finished = 3,
}

impl AgentMeta {
    /// Create default metadata for a newly-created Agent process.
    #[allow(unused)]
    pub fn new(agent_type: usize, heartbeat_interval: usize, resource_quota: usize) -> Self {
        Self {
            agent_type,
            heartbeat_interval,
            heartbeat_next_at: 0,
            pending_wake_reason: 0,
            pending_messages: 0,
            resource_quota,
            loop_state: AgentLoopState::Ready,
            context_path_meta: 0,
            context_node_count: 0,
            context_write_offset: 0,
            context_active_node: 0,
            context_next_node: 1,
            context_nodes: [ContextNode::empty(); CONTEXT_MAX_NODES],
            agent_context_base: AGENT_CONTEXT_BASE,
            agent_context_size: AGENT_CONTEXT_SIZE,
        }
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    agent: None,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let agent = self.inner_exclusive_access().agent;
        if agent.is_some() {
            assert!(memory_set.insert_framed_area_checked(
                AGENT_CONTEXT_BASE.into(),
                (AGENT_CONTEXT_BASE + AGENT_CONTEXT_SIZE).into(),
                MapPermission::R | MapPermission::W | MapPermission::U,
            ));
        }
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access current TCB exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        inner.agent = agent;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        *inner.get_trap_cx() = trap_cx;
        // **** release current PCB
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- hold parent PCB lock
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let mut memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        if parent_inner.agent.is_some() {
            memory_set.remove_area_with_start_vpn(VirtAddr::from(AGENT_CONTEXT_BASE).floor());
        }
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    agent: None,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access child PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // **** release child PCB
        // ---- release parent PCB
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
    Zombie,
}
