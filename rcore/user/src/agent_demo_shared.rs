#[macro_use]
extern crate user_lib;

use core::mem::size_of;
use core::ptr::read_volatile;
use user_lib::{
    AGENT_WAKE_HEARTBEAT, AGENT_WAKE_MESSAGE, AgentInfo, ContextNode, ContextPushRequest,
    ContextQueryRequest, ContextQueryResult, FILE_PATH_MAX_LEN, FileAttrSetRequest, OpenFlags,
    TOOL_GET_SYSTEM_STATUS, TOOL_PARAM_AGENT_TYPE, TOOL_PARAM_FILE_OWNER, TOOL_PARAM_FILE_TAG,
    TOOL_PARAM_FILE_TYPE, TOOL_PARAM_TARGET_PID, TOOL_QUERY_FILE, TOOL_QUERY_PROCESS,
    TOOL_SEND_MESSAGE, TOOL_VALUE_U64, ToolFileQueryResult, ToolParam, ToolProcessQueryResult,
    ToolRequest, ToolResponse, ToolSystemStatus, agent_create, agent_heartbeat_set,
    agent_heartbeat_stop, agent_info, agent_wait, close, context_push, context_query,
    file_attr_set, fork, get_time, open, sleep, tool_call, waitpid, write,
};

const ADMIN_AGENT: usize = 71;
const WORKER_AGENT: usize = 72;
const TYPE_MEMORY: usize = 1;
const TYPE_LOG: usize = 2;
const OWNER_ADMIN: usize = 31;
const OWNER_WORKER: usize = 32;
const TAG_SOCIAL: usize = 41;
const TAG_SYSTEM: usize = 42;

unsafe fn read_context<T: Copy>(info: &AgentInfo, offset: usize) -> T {
    unsafe { read_volatile((info.agent_context_base + offset) as *const T) }
}

fn create_file(path: &'static str, data: &[u8]) {
    let fd = open(path, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd >= 0);
    assert_eq!(write(fd as usize, data), data.len() as isize);
    assert_eq!(close(fd as usize), 0);
}

fn set_attr(path: &'static str, file_type: usize, owner: usize, tag: usize, priority: usize) {
    let request = FileAttrSetRequest {
        path_ptr: path.as_ptr() as usize,
        file_type,
        owner,
        tag,
        priority,
    };
    assert_eq!(file_attr_set(&request), 0);
}

fn setup_demo_files() {
    create_file("m7_a\0", b"admin social memory\n");
    create_file("m7_b\0", b"worker social memory\n");
    create_file("m7_c\0", b"worker system log\n");
    create_file("m7_d\0", b"worker social log\n");
    set_attr("m7_a\0", TYPE_MEMORY, OWNER_ADMIN, TAG_SOCIAL, 5);
    set_attr("m7_b\0", TYPE_MEMORY, OWNER_WORKER, TAG_SOCIAL, 9);
    set_attr("m7_c\0", TYPE_LOG, OWNER_WORKER, TAG_SYSTEM, 3);
    set_attr("m7_d\0", TYPE_LOG, OWNER_WORKER, TAG_SOCIAL, 4);
}

fn call_tool(info: &AgentInfo, request: &ToolRequest) -> ToolResponse {
    let mut response = ToolResponse::default();
    assert_eq!(tool_call(request, &mut response), 0);
    assert_eq!(response.status, 0);
    assert!(response.result_offset < info.agent_context_size);
    response
}

fn push_summary(tool_id: usize, request: &'static [u8], result: &'static [u8]) -> ContextNode {
    let push = ContextPushRequest {
        tool_id,
        flags: 0,
        request_ptr: request.as_ptr() as usize,
        request_len: request.len(),
        result_ptr: result.as_ptr() as usize,
        result_len: result.len(),
    };
    let mut node = ContextNode::default();
    assert_eq!(context_push(&push, &mut node), 0);
    node
}

fn query_path() -> ContextQueryResult {
    let request = ContextQueryRequest {
        start_index: 0,
        max_nodes: 8,
    };
    let mut result = ContextQueryResult::default();
    assert_eq!(context_query(&request, &mut result), 0);
    result
}

fn query_files(info: &AgentInfo) -> ToolFileQueryResult {
    let request = ToolRequest {
        tool_id: TOOL_QUERY_FILE,
        param_count: 3,
        params: [
            ToolParam {
                key_id: TOOL_PARAM_FILE_TYPE,
                value_type: TOOL_VALUE_U64,
                value: TYPE_MEMORY,
            },
            ToolParam {
                key_id: TOOL_PARAM_FILE_OWNER,
                value_type: TOOL_VALUE_U64,
                value: OWNER_WORKER,
            },
            ToolParam {
                key_id: TOOL_PARAM_FILE_TAG,
                value_type: TOOL_VALUE_U64,
                value: TAG_SOCIAL,
            },
            ToolParam::default(),
        ],
    };
    let response = call_tool(info, &request);
    assert_eq!(response.result_len, size_of::<ToolFileQueryResult>());
    unsafe { read_context(info, response.result_offset) }
}

fn send_message(info: &AgentInfo, target_pid: usize) {
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
    let response = call_tool(info, &request);
    assert!(response.result_len > 0);
}

fn print_path(path: &ContextQueryResult) {
    println!(
        "context_path nodes={} active={}",
        path.total_nodes, path.active_node_id
    );
    let mut i = 0;
    while i < path.returned {
        println!(
            "  node={} prev={} tool={}",
            path.nodes[i].node_id, path.nodes[i].prev_id, path.nodes[i].tool_id
        );
        i += 1;
    }
}

fn first_path_text(result: &ToolFileQueryResult) -> [u8; FILE_PATH_MAX_LEN] {
    result.items[0].path
}

fn demo_basic() -> i32 {
    println!("agent_demo basic: start");
    assert_eq!(agent_create(ADMIN_AGENT, 0, 8192), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    let status_request = ToolRequest {
        tool_id: TOOL_GET_SYSTEM_STATUS,
        ..ToolRequest::default()
    };
    let response = call_tool(&info, &status_request);
    let status: ToolSystemStatus = unsafe { read_context(&info, response.result_offset) };
    println!(
        "system_status pid={} processes={} agents={}",
        status.current_pid, status.process_count, status.agent_count
    );
    push_summary(TOOL_GET_SYSTEM_STATUS, b"get_system_status", b"system ok");

    let process_request = ToolRequest {
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
    let response = call_tool(&info, &process_request);
    let processes: ToolProcessQueryResult = unsafe { read_context(&info, response.result_offset) };
    println!("query_process admin_matches={}", processes.total_matches);
    push_summary(TOOL_QUERY_PROCESS, b"query_process admin", b"admin found");
    print_path(&query_path());
    println!("agent_demo basic: passed");
    0
}

fn demo_loop() -> i32 {
    println!("agent_demo loop: start");
    assert_eq!(agent_create(ADMIN_AGENT, 0, 8192), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    assert_eq!(agent_heartbeat_set(30), 0);
    let start = get_time();
    let reason = agent_wait();
    let elapsed = get_time() - start;
    assert_eq!(
        reason & AGENT_WAKE_HEARTBEAT as isize,
        AGENT_WAKE_HEARTBEAT as isize
    );
    assert_eq!(agent_heartbeat_stop(), 0);
    println!("heartbeat_wake elapsed_ms={}", elapsed);
    push_summary(TOOL_GET_SYSTEM_STATUS, b"wait heartbeat", b"heartbeat wake");

    let pid = fork();
    if pid == 0 {
        assert_eq!(agent_create(WORKER_AGENT, 0, 4096), 0);
        let reason = agent_wait();
        assert_eq!(
            reason & AGENT_WAKE_MESSAGE as isize,
            AGENT_WAKE_MESSAGE as isize
        );
        println!("worker woke by message");
        return 0;
    }
    sleep(40);
    send_message(&info, pid as usize);
    let mut exit_code = 0;
    assert_eq!(waitpid(pid as usize, &mut exit_code), pid);
    assert_eq!(exit_code, 0);
    println!("message_wake worker_pid={}", pid);
    push_summary(TOOL_SEND_MESSAGE, b"send_message worker", b"worker woke");
    print_path(&query_path());
    println!("agent_demo loop: passed");
    0
}

fn demo_fs_query_bench() -> i32 {
    println!("agent_demo fs_query_bench: start");
    setup_demo_files();
    assert_eq!(agent_create(ADMIN_AGENT, 0, 8192), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);
    let result = query_files(&info);
    assert_eq!(result.total_matches, 1);
    assert!(result.traversal_visits > result.indexed_visits);
    let path = first_path_text(&result);
    println!(
        "query_file matches={} traversal={} indexed={} first={}{}{}{}",
        result.total_matches,
        result.traversal_visits,
        result.indexed_visits,
        path[0] as char,
        path[1] as char,
        path[2] as char,
        path[3] as char
    );
    push_summary(TOOL_QUERY_FILE, b"query_file worker memory", b"m7_b");
    print_path(&query_path());
    println!("agent_demo fs_query_bench: passed");
    0
}

fn demo_full() -> i32 {
    println!("agent_demo full: start");
    setup_demo_files();
    assert_eq!(agent_create(ADMIN_AGENT, 0, 12288), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    let pid = fork();
    if pid == 0 {
        assert_eq!(agent_create(WORKER_AGENT, 0, 4096), 0);
        let reason = agent_wait();
        assert_eq!(
            reason & AGENT_WAKE_MESSAGE as isize,
            AGENT_WAKE_MESSAGE as isize
        );
        println!("full worker handled message");
        return 0;
    }

    assert_eq!(agent_heartbeat_set(25), 0);
    let reason = agent_wait();
    assert_eq!(
        reason & AGENT_WAKE_HEARTBEAT as isize,
        AGENT_WAKE_HEARTBEAT as isize
    );
    assert_eq!(agent_heartbeat_stop(), 0);
    println!("full heartbeat tick");
    push_summary(TOOL_GET_SYSTEM_STATUS, b"heartbeat", b"admin loop tick");

    let status_request = ToolRequest {
        tool_id: TOOL_GET_SYSTEM_STATUS,
        ..ToolRequest::default()
    };
    let response = call_tool(&info, &status_request);
    let status: ToolSystemStatus = unsafe { read_context(&info, response.result_offset) };
    println!(
        "full status processes={} agents={}",
        status.process_count, status.agent_count
    );
    push_summary(TOOL_GET_SYSTEM_STATUS, b"get_system_status", b"status cached");

    let result = query_files(&info);
    println!(
        "full file_query matches={} traversal={} indexed={}",
        result.total_matches, result.traversal_visits, result.indexed_visits
    );
    push_summary(TOOL_QUERY_FILE, b"query_file memory owner worker", b"m7_b");

    send_message(&info, pid as usize);
    let mut exit_code = 0;
    assert_eq!(waitpid(pid as usize, &mut exit_code), pid);
    assert_eq!(exit_code, 0);
    println!("full message delivered worker_pid={}", pid);
    push_summary(TOOL_SEND_MESSAGE, b"send_message worker", b"worker ack");

    print_path(&query_path());
    println!("agent_demo full: passed");
    0
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    match DEMO_MODE {
        0 => demo_basic(),
        1 => demo_loop(),
        2 => demo_fs_query_bench(),
        _ => demo_full(),
    }
}
