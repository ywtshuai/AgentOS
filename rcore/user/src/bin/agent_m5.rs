#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{
    AGENT_WAKE_HEARTBEAT, AGENT_WAKE_MESSAGE, AgentInfo, TOOL_PARAM_TARGET_PID, TOOL_SEND_MESSAGE,
    TOOL_VALUE_U64, ToolParam, ToolRequest, ToolResponse, agent_create, agent_heartbeat_set,
    agent_heartbeat_stop, agent_info, agent_wait, fork, sleep, tool_call, waitpid,
};

const ADMIN_AGENT: usize = 55;
const WORKER_AGENT: usize = 56;

fn send_message(target_pid: usize) {
    let request = ToolRequest {
        tool_id: TOOL_SEND_MESSAGE,
        param_count: 1,
        params: [
            ToolParam {
                key_id: TOOL_PARAM_TARGET_PID,
                value_type: TOOL_VALUE_U64,
                value: target_pid,
            },
            ToolParam::default(),
            ToolParam::default(),
            ToolParam::default(),
        ],
    };
    let mut response = ToolResponse::default();
    assert_eq!(tool_call(&request, &mut response), 0);
    assert_eq!(response.status, 0);
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    assert_eq!(agent_wait(), -3);
    assert_eq!(agent_heartbeat_set(10), -3);
    assert_eq!(agent_heartbeat_stop(), -3);
    println!("agent_m5 non-agent guard passed");

    assert_eq!(agent_create(ADMIN_AGENT, 0, 4096), 0);
    assert_eq!(agent_heartbeat_set(30), 0);
    let start = user_lib::get_time();
    let reason = agent_wait();
    let elapsed = user_lib::get_time() - start;
    assert_eq!(
        reason & AGENT_WAKE_HEARTBEAT as isize,
        AGENT_WAKE_HEARTBEAT as isize
    );
    assert!(elapsed >= 20);
    assert_eq!(agent_heartbeat_stop(), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);
    assert_eq!(info.heartbeat_interval, 0);
    println!("agent_m5 heartbeat wake passed");

    let pid = fork();
    if pid == 0 {
        assert_eq!(agent_create(WORKER_AGENT, 0, 1024), 0);
        let reason = agent_wait();
        assert_eq!(
            reason & AGENT_WAKE_MESSAGE as isize,
            AGENT_WAKE_MESSAGE as isize
        );
        let mut info = AgentInfo::default();
        assert_eq!(agent_info(-1, &mut info), 0);
        assert_eq!(info.pending_messages, 0);
        println!("agent_m5 worker message wake passed");
        return 0;
    }

    sleep(40);
    send_message(pid as usize);
    let mut exit_code = 0;
    assert_eq!(waitpid(pid as usize, &mut exit_code), pid);
    assert_eq!(exit_code, 0);
    println!("agent_m5 message wake passed");

    println!("agent_m5 passed");
    0
}
