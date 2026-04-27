#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{agent_info, AgentInfo};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);
    assert_eq!(info.agent_type, 7);
    assert_eq!(info.agent_context_size, 64 * 1024);

    let context = info.agent_context_base as *mut u8;
    unsafe {
        context.write_volatile(0x41);
        assert_eq!(context.read_volatile(), 0x41);
        context.add(info.agent_context_size - 1).write_volatile(0x5a);
        assert_eq!(context.add(info.agent_context_size - 1).read_volatile(), 0x5a);
    }
    println!("agent_m1_child passed");
    0
}
