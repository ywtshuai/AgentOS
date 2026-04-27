#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{agent_create, agent_info, waitpid, AgentInfo};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), -2);

    let pid = agent_create("agent_m1_child\0", 7, 100, 64 * 1024);
    assert!(pid > 0);

    assert_eq!(agent_info(pid, &mut info), 0);
    assert_eq!(info.pid, pid as usize);
    assert_eq!(info.agent_type, 7);
    assert_eq!(info.heartbeat_interval, 100);
    assert_eq!(info.resource_quota, 64 * 1024);
    assert_eq!(info.agent_context_size, 64 * 1024);

    let mut exit_code = 0;
    let wait_pid = waitpid(pid as usize, &mut exit_code);
    assert_eq!(wait_pid, pid);
    assert_eq!(exit_code, 0);
    println!("agent_m1 passed");
    0
}
