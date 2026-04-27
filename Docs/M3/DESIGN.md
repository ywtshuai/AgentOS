# M3 Design

## 目标

M3 实现结构化 Tool Call 接口，让 Agent 进程通过固定二进制协议调用内核工具，内核执行后把结构化结果写入 Agent Context 区。

本里程碑对应 `Docs/PLAN.md` 中的“结构化 Tool Call 接口”。

## 协议

请求结构：

```text
ToolRequest {
  tool_id,
  param_count,
  params[4] = ToolParam { key_id, value_type, value }
}
```

响应结构：

```text
ToolResponse {
  status,
  result_len,
  result_offset
}
```

结果不直接通过 syscall 返回大对象，而是写入当前 Agent 的 Agent Context 区。用户态通过 `agent_info()` 得到 `agent_context_base` 后，用 `result_offset` 直接读取结构体结果。

## 工具集

M3 提供三个内核工具：

- `TOOL_GET_SYSTEM_STATUS = 1`
  - 无参数。
  - 返回进程数、Agent 数、状态统计、当前 pid 和时间戳。
- `TOOL_QUERY_PROCESS = 2`
  - 支持按 `status`、`agent_type` 过滤。
  - 返回最多 8 条进程摘要，并保留总匹配数。
- `TOOL_SEND_MESSAGE = 3`
  - 参数为 `target_pid`。
  - 当前版本验证目标 pid 是 Agent，并返回结构化确认结果；真正的消息队列和唤醒语义留到 Agent Loop 里程碑实现。

## 错误码

```text
 0  OK
-3  非 Agent 进程调用 Agent 专属 tool_call
-4  未知 tool_id
-5  参数数量或参数类型错误
-6  Agent Context 配额不足
-7  目标对象不存在或目标不是 Agent
```

## Context 写入策略

M3 使用 `AgentMeta.context_path_meta` 作为 Tool Result 写入游标：

- 写入范围受 `resource_quota` 和 `agent_context_size` 约束。
- 单次结果大于配额时返回 `-6`。
- 剩余空间不足时从 offset `0` 重新写入。

这只是工具结果缓存策略。M4 的 Context Path 会在此基础上定义正式节点布局和淘汰规则。

## Fork 语义修正

M2 已定义 `fork()` 子进程默认不是 Agent。M3 测试发现：若父进程是 Agent，子进程页表会复制 Agent Context 映射，但 PCB 元数据不是 Agent，导致子进程后续 `agent_create()` 映射冲突。

本里程碑修正为：Agent 父进程 `fork()` 时，普通子进程会移除复制来的 Agent Context 区，保持“元数据非 Agent”和“地址空间非 Agent”一致。
