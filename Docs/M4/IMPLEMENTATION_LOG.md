# M4 Implementation Log

## 修改内容

- 扩展 `AgentMeta`：
  - 增加 Context Path 节点数、写入 offset、active node、next node id。
  - 增加固定 16 项的 `ContextNode` 元数据数组。
- 新增用户/内核共享结构：
  - `ContextNode`
  - `ContextPushRequest`
  - `ContextQueryRequest`
  - `ContextQueryResult`
- 新增 syscall id：
  - `504`: `sys_context_push`
  - `505`: `sys_context_query`
  - `506`: `sys_context_rollback`
  - `507`: `sys_context_clear`
- `context_push` 将请求摘要、结果摘要和节点元数据写入 Agent Context 区，并更新 PCB 元信息。
- `context_query` 返回当前路径元数据和最多 8 个节点。
- `context_rollback` 将 active 节点回退到指定历史节点，并截断之后的 live 节点。
- `context_clear` 清空路径元信息并重置写入 offset。
- M3 Tool Call 结果写入改为使用新的 `context_write_offset`，避免继续把 `context_path_meta` 当字节游标。
- 新增用户态测试程序 `agent_m4`。

## 关键文件

- `rcore/os/src/task/task.rs`
- `rcore/os/src/task/mod.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m4.rs`

## 行为说明

`context_push` 的写入顺序是 request summary、result summary、node metadata。返回的 `ContextNode` 中包含 offset 和 len，用户态可直接从 `agent_context_base + offset` 读取摘要内容。

当新节点无法放入剩余 quota 时，内核执行简化 FIFO：写入 offset 回到 `0`，旧 live 节点元数据被淘汰，新节点成为新的 active 节点。

## 已知限制

- 当前没有为 Context Path 单独划分工具结果缓存区，二者共享同一个 Context 写入游标。
- FIFO 淘汰粒度较粗，换轮时会清空旧 live 节点；后续可升级为真正的 byte ring + node ring。
- Context Path 还未与 M5 的 Agent Loop 自动绑定，M4 先提供显式 syscall。
