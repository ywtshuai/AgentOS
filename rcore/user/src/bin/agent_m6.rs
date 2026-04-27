#![no_std]
#![no_main]

extern crate user_lib;

use core::mem::size_of;
use core::ptr::read_volatile;
use user_lib::{
    AgentInfo, FileAttrSetRequest, OpenFlags, TOOL_PARAM_FILE_OWNER, TOOL_PARAM_FILE_TAG,
    TOOL_PARAM_FILE_TYPE, TOOL_QUERY_FILE, TOOL_VALUE_U64, ToolFileQueryResult, ToolParam,
    ToolRequest, ToolResponse, agent_create, agent_info, close, file_attr_delete, file_attr_set,
    open, println, tool_call, write,
};

const AGENT_TYPE: usize = 61;
const TYPE_MEMORY: usize = 1;
const TYPE_LOG: usize = 2;
const OWNER_AGENT_A: usize = 11;
const OWNER_AGENT_B: usize = 12;
const TAG_SOCIAL: usize = 21;
const TAG_SYSTEM: usize = 22;

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

fn query_files(info: &AgentInfo, owner: usize, tag: usize) -> ToolFileQueryResult {
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
                value: owner,
            },
            ToolParam {
                key_id: TOOL_PARAM_FILE_TAG,
                value_type: TOOL_VALUE_U64,
                value: tag,
            },
            ToolParam::default(),
        ],
    };
    let mut response = ToolResponse::default();
    assert_eq!(tool_call(&request, &mut response), 0);
    assert_eq!(response.status, 0);
    assert_eq!(response.result_len, size_of::<ToolFileQueryResult>());
    unsafe { read_context(info, response.result_offset) }
}

#[unsafe(no_mangle)]
fn main() -> i32 {
    create_file("m6_a\0", b"agent-a social memory\n");
    create_file("m6_b\0", b"agent-b social memory\n");
    create_file("m6_c\0", b"agent-b system log\n");
    create_file("m6_d\0", b"agent-b social log\n");

    assert_eq!(agent_create(AGENT_TYPE, 0, 8192), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    set_attr("m6_a\0", TYPE_MEMORY, OWNER_AGENT_A, TAG_SOCIAL, 5);
    set_attr("m6_b\0", TYPE_MEMORY, OWNER_AGENT_B, TAG_SOCIAL, 9);
    set_attr("m6_c\0", TYPE_LOG, OWNER_AGENT_B, TAG_SYSTEM, 3);
    set_attr("m6_d\0", TYPE_LOG, OWNER_AGENT_B, TAG_SOCIAL, 4);
    println!("agent_m6 file attrs set passed");

    let result = query_files(&info, OWNER_AGENT_B, TAG_SOCIAL);
    assert_eq!(result.total_matches, 1);
    assert_eq!(result.returned, 1);
    assert_eq!(result.items[0].file_type, TYPE_MEMORY);
    assert_eq!(result.items[0].owner, OWNER_AGENT_B);
    assert_eq!(result.items[0].tag, TAG_SOCIAL);
    assert_eq!(result.items[0].path_len, 4);
    assert_eq!(result.items[0].path[0], b'm');
    assert_eq!(result.items[0].path[3], b'b');
    println!(
        "agent_m6 query_file passed traversal={} indexed={}",
        result.traversal_visits, result.indexed_visits
    );
    assert!(result.traversal_visits > result.indexed_visits);

    assert_eq!(file_attr_delete("m6_b\0"), 0);
    let after_delete = query_files(&info, OWNER_AGENT_B, TAG_SOCIAL);
    assert_eq!(after_delete.total_matches, 0);
    println!("agent_m6 delete passed");

    println!("agent_m6 passed");
    0
}
