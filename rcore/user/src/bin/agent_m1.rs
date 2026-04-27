#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{AgentInfo, agent_info, fork, waitpid};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), -2);

    let pid = fork();
    if pid == 0 {
        assert_eq!(agent_info(-1, &mut info), -2);
        println!("agent_m1 child passed");
        return 0;
    }

    assert_eq!(agent_info(pid, &mut info), -2);

    let mut exit_code = 0;
    let wait_pid = waitpid(pid as usize, &mut exit_code);
    assert_eq!(wait_pid, pid);
    assert_eq!(exit_code, 0);
    println!("agent_m1 passed");
    0
}
