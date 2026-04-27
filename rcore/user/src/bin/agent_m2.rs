#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::ptr::{read_volatile, write_volatile};
use user_lib::{AGENT_CONTEXT_SIZE, AgentInfo, agent_create, agent_info, fork, waitpid};

const AGENT_TYPE_TEST: usize = 7;
const HEARTBEAT_MS: usize = 250;
const QUOTA_BYTES: usize = 4096;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), -2);

    assert_eq!(agent_create(AGENT_TYPE_TEST, HEARTBEAT_MS, QUOTA_BYTES), 0);
    println!("agent_m2 create passed");
    assert_eq!(agent_info(-1, &mut info), 0);
    println!(
        "agent_m2 context base={:#x} size={}",
        info.agent_context_base, info.agent_context_size
    );
    assert_eq!(info.agent_type, AGENT_TYPE_TEST);
    assert_eq!(info.heartbeat_interval, HEARTBEAT_MS);
    assert_eq!(info.resource_quota, QUOTA_BYTES);
    assert_eq!(info.agent_context_size, AGENT_CONTEXT_SIZE);

    let context = info.agent_context_base as *mut u8;
    unsafe {
        write_volatile(context, 0x41);
        write_volatile(context.add(info.agent_context_size - 1), 0x5a);
        assert_eq!(read_volatile(context), 0x41);
        assert_eq!(
            read_volatile(context.add(info.agent_context_size - 1)),
            0x5a
        );
    }
    println!("agent_m2 context rw passed");

    assert_eq!(agent_create(AGENT_TYPE_TEST, HEARTBEAT_MS, QUOTA_BYTES), -1);
    println!("agent_m2 duplicate create passed");

    let pid = fork();
    if pid == 0 {
        let mut child_info = AgentInfo::default();
        assert_eq!(agent_info(-1, &mut child_info), -2);
        println!("agent_m2 child passed");
        return 0;
    }
    let mut exit_code = 0;
    assert_eq!(waitpid(pid as usize, &mut exit_code), pid);
    assert_eq!(exit_code, 0);

    println!("agent_m2 passed");
    0
}
