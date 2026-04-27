# M6 Implementation Log

## 修改内容

- 新增内核内存态文件属性表，容量 16 项。
- 新增结构体：
  - `FileAttrSetRequest`
  - `ToolFileSummary`
  - `ToolFileQueryResult`
- 新增 syscall id：
  - `511`: `sys_file_attr_set`
  - `512`: `sys_file_attr_delete`
- 新增 Tool Call：
  - `TOOL_QUERY_FILE = 4`
- 新增文件属性查询参数：
  - `TOOL_PARAM_FILE_TYPE`
  - `TOOL_PARAM_FILE_OWNER`
  - `TOOL_PARAM_FILE_TAG`
  - `TOOL_PARAM_FILE_PRIORITY`
- `query_file` 支持多条件 AND 查询，返回结构化文件摘要和访问次数对比。
- 更新用户态 syscall/lib 封装，暴露文件属性请求和查询结果结构。
- 新增用户态测试程序 `agent_m6`。
- 调整 `agent_m3`，允许 `tool_list()` 返回新增工具后的数量。

## 关键文件

- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m3.rs`
- `rcore/user/src/bin/agent_m6.rs`

## 行为说明

`agent_m6` 会创建 4 个测试文件，为它们绑定属性，然后以 Agent 身份调用 `query_file(type=memory AND owner=agent-b AND tag=social)`。查询结果只返回 `m6_b`，并输出：

```text
traversal=4 indexed=2
```

这表示全量遍历需要检查 4 个属性项，而按第一个查询条件 `type=memory` 作为索引键后只需要检查 2 个候选项。

## 已知限制

- 属性表是内存态，重启后不保留。
- 路径最长 32 字节，表容量为 16 项。
- 当前索引是简化的候选项访问模型，没有维护独立哈希桶；足以支撑 M6 的演示和性能对比输出。
