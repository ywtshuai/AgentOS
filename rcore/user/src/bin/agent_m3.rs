#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::mem::size_of;
use core::ptr::read_volatile;
use user_lib::{
    AgentInfo, TOOL_GET_SYSTEM_STATUS, TOOL_PARAM_AGENT_TYPE, TOOL_PARAM_TARGET_PID,
    TOOL_QUERY_PROCESS, TOOL_SEND_MESSAGE, TOOL_VALUE_U64, ToolInfo, ToolMessageResult, ToolParam,
    ToolProcessQueryResult, ToolRequest, ToolResponse, ToolSystemStatus, agent_create, agent_info,
    fork, sleep, tool_call, tool_list, waitpid,
};

const ADMIN_AGENT: usize = 11;
const WORKER_AGENT: usize = 22;

unsafe fn read_context<T: Copy>(info: &AgentInfo, offset: usize) -> T {
    unsafe { read_volatile((info.agent_context_base + offset) as *const T) }
}

fn call_tool(request: &ToolRequest, info: &AgentInfo) -> ToolResponse {
    let mut response = ToolResponse::default();
    assert_eq!(tool_call(request, &mut response), 0);
    assert_eq!(response.status, 0);
    assert!(response.result_offset < info.agent_context_size);
    response
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut tools = [ToolInfo::default(); 4];
    assert_eq!(tool_list(&mut tools), 3);
    assert_eq!(tools[0].tool_id, TOOL_GET_SYSTEM_STATUS);
    assert_eq!(tools[1].tool_id, TOOL_QUERY_PROCESS);
    assert_eq!(tools[2].tool_id, TOOL_SEND_MESSAGE);
    println!("agent_m3 tool_list passed");

    let mut response = ToolResponse::default();
    let request = ToolRequest {
        tool_id: TOOL_GET_SYSTEM_STATUS,
        ..ToolRequest::default()
    };
    assert_eq!(tool_call(&request, &mut response), -3);
    assert_eq!(response.status, -3);
    println!("agent_m3 non-agent guard passed");

    assert_eq!(agent_create(ADMIN_AGENT, 100, 4096), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    let response = call_tool(&request, &info);
    assert_eq!(response.result_len, size_of::<ToolSystemStatus>());
    let status: ToolSystemStatus = unsafe { read_context(&info, response.result_offset) };
    assert!(status.process_count >= 1);
    assert!(status.agent_count >= 1);
    assert_eq!(status.current_pid, info.pid);
    println!("agent_m3 get_system_status passed");

    let query = ToolRequest {
        tool_id: TOOL_QUERY_PROCESS,
        param_count: 1,
        params: [
            ToolParam {
                key_id: TOOL_PARAM_AGENT_TYPE,
                value_type: TOOL_VALUE_U64,
                value: ADMIN_AGENT,
            },
            ToolParam::default(),
            ToolParam::default(),
            ToolParam::default(),
        ],
    };
    let response = call_tool(&query, &info);
    assert_eq!(response.result_len, size_of::<ToolProcessQueryResult>());
    let processes: ToolProcessQueryResult = unsafe { read_context(&info, response.result_offset) };
    assert!(processes.total_matches >= 1);
    assert!(processes.returned >= 1);
    assert_eq!(processes.items[0].agent_type, ADMIN_AGENT);
    println!("agent_m3 query_process passed");

    let pid = fork();
    if pid == 0 {
        assert_eq!(agent_create(WORKER_AGENT, 0, 1024), 0);
        sleep(80);
        return 0;
    }
    sleep(20);

    let send = ToolRequest {
        tool_id: TOOL_SEND_MESSAGE,
        param_count: 1,
        params: [
            ToolParam {
                key_id: TOOL_PARAM_TARGET_PID,
                value_type: TOOL_VALUE_U64,
                value: pid as usize,
            },
            ToolParam::default(),
            ToolParam::default(),
            ToolParam::default(),
        ],
    };
    let mut response = call_tool(&send, &info);
    assert_eq!(response.result_len, size_of::<ToolMessageResult>());
    let message: ToolMessageResult = unsafe { read_context(&info, response.result_offset) };
    assert_eq!(message.target_pid, pid as usize);
    assert_eq!(message.target_agent_type, WORKER_AGENT);
    assert_eq!(message.accepted, 1);
    let mut exit_code = 0;
    assert_eq!(waitpid(pid as usize, &mut exit_code), pid);
    assert_eq!(exit_code, 0);
    println!("agent_m3 send_message passed");

    let bad = ToolRequest {
        tool_id: 99,
        ..ToolRequest::default()
    };
    assert_eq!(tool_call(&bad, &mut response), -4);
    assert_eq!(response.status, -4);
    println!("agent_m3 bad_tool passed");

    println!("agent_m3 passed");
    0
}
