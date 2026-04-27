# M4 Design

## 目标

M4 实现 Context Path 管理，对应 `Docs/PLAN.md` 中的“Context Path 管理”：

- Agent 可把每轮工具调用的请求摘要和结果摘要追加为 Context Path 节点。
- 内核维护节点数、写入位置、active 节点和 FIFO 元数据。
- 节点正文存放在用户态 Agent Context 区，Agent 可直接读取。
- 支持 query、rollback、clear。

## 存储布局

M4 继续复用 M2 的 64KB Agent Context 区。

每次 `context_push` 按顺序写入：

```text
request summary bytes
result summary bytes
ContextNode metadata
```

`ContextNode` 包含：

- `node_id`
- `prev_id`
- `timestamp`
- `tool_id`
- `request_offset/request_len`
- `result_offset/result_len`
- `node_offset`
- `flags`

请求和结果摘要在 Agent Context 中直接按 offset 读取；内核 PCB 中只保存最多 16 个近期节点的元数据索引。

## 内核元信息

`AgentMeta` 新增：

- `context_node_count`
- `context_write_offset`
- `context_active_node`
- `context_next_node`
- `context_nodes[16]`

`AgentInfo.context_path_meta` 保留为兼容字段，M4 中表示当前 live node 数。

## 系统调用

新增 syscall：

- `504`: `sys_context_push(push_request, out_node)`
- `505`: `sys_context_query(query_request, out_result)`
- `506`: `sys_context_rollback(node_id)`
- `507`: `sys_context_clear()`

错误码沿用 M3 Agent 接口：

- `0`: OK
- `-3`: 当前进程不是 Agent
- `-6`: 单个节点超过 Context 配额
- `-7`: rollback 目标节点不存在

## 配额与淘汰

写入空间受 `resource_quota` 限制；若 quota 为 `0` 或大于 Context 大小，则使用完整 64KB。

当剩余空间不足以写入新节点时，写入游标回到 offset `0`，旧节点元数据被清空，新节点成为新的路径起点。这是 M4 的简化 FIFO 淘汰策略，保证 Context 不会无限增长。

## 已知限制

- M4 的 FIFO 在环形空间换轮时会清空当前 live 节点，而不是逐节点精细回收。
- Context Path 节点上限为 16，`context_query` 单次最多返回 8 个节点。
- Tool Call 结果缓存和 Context Path 共用 Agent Context 写入游标；M5/M6 综合 demo 可以在用户态决定哪些工具结果再显式 push 成路径节点。
