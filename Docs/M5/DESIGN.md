# M5 Design

## 目标

M5 实现 Agent Loop 的最小内核运行机制，对应 `Docs/PLAN.md` 中的 “Agent Loop Wakeup”：

- Agent 可设置或停止心跳周期。
- Agent 可调用 `sys_agent_wait` 进入等待态，不在 ready queue 中持续占用 CPU。
- timer interrupt 到达心跳期限后唤醒等待中的 Agent。
- `send_message` 工具向目标 Agent 发送结构化消息时唤醒目标 Agent。

## 调度状态

`TaskStatus` 新增：

- `Blocked`

`block_current_and_run_next()` 会把当前任务状态改为 `Blocked`，保存任务上下文后切回调度器，但不会把任务重新放回 ready queue。

`wake_agent_by_pid(pid, reason)` 会记录唤醒原因；如果目标任务处于 `Blocked`，则把它改为 `Ready` 并重新加入 ready queue。

## Agent 元数据

`AgentMeta` 新增：

- `heartbeat_next_at`：下一次心跳到期时间，单位 ms，`0` 表示未 armed。
- `pending_wake_reason`：等待返回的唤醒原因 bitset。
- `pending_messages`：待消费的结构化消息数量。

唤醒原因：

- `AGENT_WAKE_HEARTBEAT = 1`
- `AGENT_WAKE_MESSAGE = 2`

`AgentInfo` 同步暴露这些字段，方便用户态测试和 demo 查询 Agent Loop 状态。

## 系统调用

新增 syscall：

- `508`: `sys_agent_heartbeat_set(interval_ms)`
- `509`: `sys_agent_heartbeat_stop()`
- `510`: `sys_agent_wait()`

返回值：

- `0`: 设置/停止成功。
- `-3`: 当前进程不是 Agent。
- `sys_agent_wait()` 被唤醒后返回 wake reason bitset。

## 心跳唤醒

timer interrupt 每 tick 调用 `check_agent_heartbeats()`，从 `INITPROC` 进程树扫描 Agent：

1. 若 `heartbeat_interval > 0` 且当前时间到达 `heartbeat_next_at`，写入 `AGENT_WAKE_HEARTBEAT`。
2. 更新下一次 deadline 为 `now + heartbeat_interval`。
3. 若 Agent 正在 `Blocked`，将其切回 `Ready`。

## 消息唤醒

M3 的 `send_message` 工具在 M5 中扩展为：

1. 校验目标 pid 存在且是 Agent。
2. 增加目标 Agent 的 `pending_messages`。
3. 写入 `AGENT_WAKE_MESSAGE`。
4. 如果目标 Agent 正在等待，则唤醒它。

消息正文仍保持 M3 的简化结构化确认；M5 只实现“消息事件触发 Agent Loop”的内核机制，完整消息队列可在后续综合 demo 中扩展。

## 已知限制

- 心跳扫描采用进程树线性遍历，适合当前教学 demo；大量 Agent 时可升级为 deadline queue。
- `pending_messages` 当前只记录计数，不保存多条消息正文。
- `sys_agent_wait` 返回 bitset，若心跳和消息同时到达，用户态需要自行按 bit 判断。
