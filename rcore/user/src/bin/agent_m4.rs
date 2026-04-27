#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::ptr::read_volatile;
use user_lib::{
    AgentInfo, ContextNode, ContextPushRequest, ContextQueryRequest, ContextQueryResult,
    agent_create, agent_info, context_clear, context_push, context_query, context_rollback,
};

const AGENT_TYPE: usize = 44;
const QUOTA: usize = 1024;

unsafe fn read_context_byte(info: &AgentInfo, offset: usize) -> u8 {
    unsafe { read_volatile((info.agent_context_base + offset) as *const u8) }
}

fn push_node(tool_id: usize, request: &[u8], result: &[u8]) -> ContextNode {
    let push = ContextPushRequest {
        tool_id,
        flags: tool_id + 100,
        request_ptr: request.as_ptr() as usize,
        request_len: request.len(),
        result_ptr: result.as_ptr() as usize,
        result_len: result.len(),
    };
    let mut node = ContextNode::default();
    assert_eq!(context_push(&push, &mut node), 0);
    assert_eq!(node.tool_id, tool_id);
    assert_eq!(node.request_len, request.len());
    assert_eq!(node.result_len, result.len());
    node
}

fn query_all() -> ContextQueryResult {
    let request = ContextQueryRequest {
        start_index: 0,
        max_nodes: 8,
    };
    let mut result = ContextQueryResult::default();
    assert_eq!(context_query(&request, &mut result), 0);
    result
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let request = ContextQueryRequest::default();
    let mut query = ContextQueryResult::default();
    assert_eq!(context_query(&request, &mut query), -3);
    println!("agent_m4 non-agent guard passed");

    assert_eq!(agent_create(AGENT_TYPE, 0, QUOTA), 0);
    let mut info = AgentInfo::default();
    assert_eq!(agent_info(-1, &mut info), 0);

    let mut saved = [ContextNode::default(); 5];
    let mut i = 0;
    while i < 5 {
        let req = [b'a' + i as u8, b'r'];
        let res = [b'A' + i as u8, b'R', b'0'];
        saved[i] = push_node(i + 1, &req, &res);
        if i == 0 {
            assert_eq!(saved[i].prev_id, 0);
        } else {
            assert_eq!(saved[i].prev_id, saved[i - 1].node_id);
        }
        assert_eq!(
            unsafe { read_context_byte(&info, saved[i].request_offset) },
            req[0]
        );
        assert_eq!(
            unsafe { read_context_byte(&info, saved[i].result_offset) },
            res[0]
        );
        i += 1;
    }

    query = query_all();
    assert_eq!(query.total_nodes, 5);
    assert_eq!(query.returned, 5);
    assert_eq!(query.active_node_id, saved[4].node_id);
    assert_eq!(query.nodes[2].node_id, saved[2].node_id);
    println!("agent_m4 push/query passed");

    assert_eq!(context_rollback(saved[2].node_id), 0);
    query = query_all();
    assert_eq!(query.total_nodes, 3);
    assert_eq!(query.active_node_id, saved[2].node_id);
    println!("agent_m4 rollback passed");

    let big_req = [7u8; 900];
    let big_res = [9u8; 1];
    let big = push_node(99, &big_req, &big_res);
    query = query_all();
    assert_eq!(query.total_nodes, 1);
    assert_eq!(query.active_node_id, big.node_id);
    assert_eq!(big.request_offset, 0);
    println!("agent_m4 fifo wrap passed");

    assert_eq!(context_clear(), 0);
    query = query_all();
    assert_eq!(query.total_nodes, 0);
    assert_eq!(query.active_node_id, 0);
    println!("agent_m4 clear passed");

    println!("agent_m4 passed");
    0
}
