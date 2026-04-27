# M5 Implementation Log

## 修改内容

- 扩展 `TaskStatus`，新增 `Blocked`。
- 新增任务调度辅助：
  - `block_current_and_run_next()`
  - `check_agent_heartbeats()`
  - `wake_agent_by_pid()`
- 扩展 `AgentMeta`：
  - `heartbeat_next_at`
  - `pending_wake_reason`
  - `pending_messages`
- 扩展 `AgentInfo`，向用户态暴露心跳 deadline、pending wake reason 和 pending message 计数。
- timer interrupt 中接入 `check_agent_heartbeats()`，心跳到期时唤醒等待中的 Agent。
- 新增 syscall id：
  - `508`: `sys_agent_heartbeat_set`
  - `509`: `sys_agent_heartbeat_stop`
  - `510`: `sys_agent_wait`
- `send_message` 工具新增消息事件语义：
  - 增加目标 Agent 的 pending message 计数。
  - 写入 `AGENT_WAKE_MESSAGE` 唤醒原因。
  - 唤醒处于等待态的目标 Agent。
- 新增用户态封装：
  - `agent_heartbeat_set()`
  - `agent_heartbeat_stop()`
  - `agent_wait()`
- 新增用户态测试程序 `agent_m5`。

## 关键文件

- `rcore/os/src/task/task.rs`
- `rcore/os/src/task/mod.rs`
- `rcore/os/src/trap/mod.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m5.rs`

## 行为说明

`sys_agent_wait()` 的行为：

1. 非 Agent 调用返回 `-3`。
2. 如果已有 pending wake reason，立即消费并返回，不阻塞。
3. 否则将 Agent loop state 设置为 `Waiting`，任务状态改为 `Blocked`，切出调度。
4. 心跳或消息事件到达后，内核把任务重新放回 ready queue。
5. Agent 继续从 `sys_agent_wait()` 返回，返回值为 wake reason bitset。

心跳通过 timer interrupt 驱动，不需要用户态 busy wait。消息唤醒复用 M3 的 `TOOL_SEND_MESSAGE`，因此 Agent 间协作仍走结构化 Tool Call 接口。

## 已知限制

- 当前唤醒原因是 bitset，没有单独的事件队列；多次消息会合并唤醒原因，但 `pending_messages` 保留计数。
- `ToolSystemStatus` 暂未新增 blocked 计数，为保持 M3 用户态结构体兼容，Blocked 任务不计入 ready/running/zombie。
- 心跳管理还不是高性能 timer wheel/deadline heap，M5 优先保证演示稳定和实现清晰。
