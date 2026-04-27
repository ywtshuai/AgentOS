# Agent-OS 赛题实现计划

## Summary

目标是做一套可在 QEMU 上运行的教学 OS 扩展系统，优先保证基础任务完整拿分，再用高性价比进阶功能做出可演示亮点。

默认技术路线：

- 基于 `rCore-Tutorial v3` / RISC-V 64 / QEMU。
- 不在内核中接入真实 LLM，只实现 Agent 所需的 OS 机制。
- 用户态 Agent 程序模拟 “思考 → 工具调用 → 观察 → 记录上下文 → 下一轮决策”。
- 第一目标：完成任务一、二、三。
- 第二目标：完成任务五中的心跳与消息唤醒。
- 第三目标：实现简化版文件属性查询，支撑综合演示和性能对比。

预计节奏：

- 基础可跑版本：3-4 周。
- 完整可展示版本：5-7 周。
- 文档、性能评估、演示 polish：1 周。

## Key Changes

### 1. Agent 进程与地址空间

- 扩展内核进程控制块，新增 Agent 元信息：
  - `agent_type`
  - `heartbeat_interval`
  - `resource_quota`
  - `loop_state`
  - `context_path_meta`
  - `agent_context_base`
  - `agent_context_size`
- 新增用户态 `Agent Context` 区：
  - 默认大小 `64KB`。
  - 映射在用户堆和用户栈之间的固定虚拟地址区间。
  - 存储上下文路径节点、工具调用历史、查询结果摘要。
- 新增系统调用：
  - `sys_agent_create`
  - `sys_agent_info`
- 验收目标：
  - Agent 进程和普通进程可共存。
  - Agent 进程能直接读写自己的 Agent Context 区。
  - 内核能查询 Agent 元信息。

### 2. 结构化 Tool Call 接口

- 使用简单二进制协议，不实现完整 JSON，降低内核解析复杂度。
- 请求结构包含：
  - `tool_id`
  - `param_count`
  - 固定长度参数数组，参数为 `key_id + value_type + value`
- 响应结构包含：
  - `status`
  - `result_len`
  - `result_offset`
  - 结果摘要写入 Agent Context 区。
- 第一批内核工具：
  - `get_system_status`：返回进程数、Agent 数、内存页统计等。
  - `query_process`：按进程状态、类型查询进程。
  - `send_message`：向另一个 Agent 发送结构化消息。
  - 可选 `query_file`：按文件属性或伪属性查询文件。
- 新增系统调用：
  - `sys_tool_call`
  - `sys_tool_list`
- 错误处理：
  - 工具不存在。
  - 参数数量错误。
  - 参数类型错误。
  - Agent Context 空间不足。
  - 非 Agent 进程调用 Agent 专属接口。

### 3. Context Path 管理

- 在 Agent Context 区实现环形缓冲区。
- 每个上下文节点记录：
  - `node_id`
  - `prev_id`
  - `timestamp`
  - `tool_id`
  - `request_summary_offset`
  - `result_summary_offset`
  - `flags`
- 内核 PCB 只维护元信息：
  - 当前节点数。
  - 当前写入位置。
  - 当前 active 节点。
  - 配额。
  - 淘汰策略。
- 默认淘汰策略：FIFO。
- 新增系统调用：
  - `sys_context_push`
  - `sys_context_query`
  - `sys_context_rollback`
  - `sys_context_clear`
- 验收目标：
  - Agent 连续执行至少 5 轮工具调用。
  - 每轮请求与结果都写入 Context Path。
  - 超出配额后自动淘汰旧节点，不导致内存无限增长。

### 4. Agent Loop 与事件触发

- 实现高性价比版本，不做复杂事件系统。
- 支持：
  - 心跳周期唤醒。
  - Agent 主动等待。
  - `send_message` 唤醒目标 Agent。
- 新增系统调用：
  - `sys_agent_heartbeat_set`
  - `sys_agent_heartbeat_stop`
  - `sys_agent_wait`
- Agent Loop 状态：
  - `Ready`
  - `Waiting`
  - `Running`
  - `Finished`
- 验收目标：
  - Agent 无事件时休眠，不持续占用 CPU。
  - 心跳到达后被唤醒。
  - 收到其他 Agent 消息后被唤醒。

### 5. 简化文件查询扩展

- 实现内存态文件属性表，先不强改磁盘 inode 格式。
- 支持为文件绑定属性：
  - `type`
  - `owner`
  - `tag`
  - `priority`
- 支持组合查询：
  - `type=memory AND owner=agent-b`
- 可选建立简单哈希索引：
  - key 为 `属性名 + 属性值`
  - value 为文件 inode/path 列表
- 对比测试：
  - 全量遍历文件查询。
  - 属性索引查询。
- 验收目标：
  - Agent 不知道完整路径时，也能通过属性找到文件。
  - 提供至少一组性能对比数据。

## Demo Plan

综合演示采用“Agent 系统管理员”场景。

演示流程：

1. 启动 QEMU。
2. 创建一个普通用户进程和两个 Agent 进程。
3. Admin-Agent 设置心跳周期。
4. Admin-Agent 每轮醒来后执行：
   - `get_system_status`
   - `query_process`
   - `query_file`
   - `send_message`
   - `context_push`
5. Worker-Agent 收到消息后被唤醒，返回状态。
6. Admin-Agent 的 Context Path 展示完整查询路径。
7. 人为制造较多 context 节点，展示 FIFO 淘汰。
8. 运行文件查询性能测试，输出属性索引查询优于遍历查询的数据。

最终演示命令建议：

```bash
make run
agent_demo basic
agent_demo loop
agent_demo fs_query_bench
agent_demo full
```

## Test Plan

基础测试：

- 普通进程创建不受 Agent 扩展影响。
- Agent 创建后 PCB 字段初始化正确。
- Agent Context 区可直接读写。
- 非 Agent 进程调用 Agent syscall 返回权限错误。

Tool Call 测试：

- `tool_list` 返回工具列表。
- 三个工具正常返回结构化结果。
- 错误 tool id 返回明确错误码。
- 参数类型错误返回明确错误码。
- 结果过大时不会越界写入 Agent Context。

Context Path 测试：

- 连续 push 5 个节点后可完整查询。
- rollback 到历史节点后 active 节点正确变化。
- clear 后路径为空。
- 超过 quota 后 FIFO 淘汰生效。

Agent Loop 测试：

- Agent wait 后进入休眠。
- 心跳到达后被唤醒。
- `send_message` 能唤醒目标 Agent。
- 多 Agent 并发运行无 panic。

文件查询测试：

- 设置文件属性后可按属性查询。
- 多条件查询结果正确。
- 索引查询和遍历查询输出对比耗时或访问次数。

## Documentation Deliverables

需要提交的文档：

- `README.md`
  - 编译方式。
  - QEMU 运行方式。
  - demo 命令。
  - 系统调用列表。
- `docs/design.md`
  - Agent-OS 总体架构。
  - Agent 进程模型。
  - 地址空间布局。
  - Tool Call 协议。
  - Context Path 设计。
  - Agent Loop 机制。
- `docs/evaluation.md`
  - 功能测试结果。
  - 文件查询性能对比。
  - Agent Context 直接读写与 syscall 查询的对比。
- 推荐包含 Mermaid 图：
  - 系统架构图。
  - Agent Loop 状态机。
  - Agent Context 内存布局。
  - Tool Call 数据流。

## Assumptions

- 默认从空项目开始搭建，当前 `e:\AgentOS` 目录未发现已有 OS 仓库内容。
- 默认选择 `rCore-Tutorial v3` 作为基础内核。
- 默认不接入真实外部 LLM API。
- 默认协议采用固定结构体或 TLV，不采用完整 JSON。
- 默认文件属性系统先以内存态实现，保证演示和评分点；如时间充足再做持久化。
- 默认综合演示选择“Agent 系统管理员”，因为稳定、容易讲清楚、工程风险低。

## Milestone Breakdown

### M0: Project Baseline

目标：

- 初始化或同步远程 Git 仓库。
- 引入 rCore-Tutorial v3 基础代码。
- 确认 Rust、QEMU、RISC-V target、构建脚本可用。
- 修正文档编码，建立里程碑文档目录规范。

交付：

- 项目能完成基础构建或明确记录缺失环境。
- `Docs/M0/DESIGN.md`
- `Docs/M0/IMPLEMENTATION_LOG.md`
- `Docs/M0/TEST_REPORT.md`

### M1: Agent Process Metadata

目标：

- 扩展 PCB / task control block，加入 Agent 元信息。
- 实现 Agent 类型标识、资源配额、Loop 状态、Context 元信息字段。
- 实现 `sys_agent_info`，用于查询当前进程或指定进程的 Agent 元信息。
- 提供用户态 M1 测试程序验证普通进程与 Agent 元信息不互相影响。

交付：

- Agent 元信息结构稳定。
- 普通进程默认不是 Agent。
- Agent 元信息查询 syscall 可用。
- `Docs/M1/DESIGN.md`
- `Docs/M1/IMPLEMENTATION_LOG.md`
- `Docs/M1/TEST_REPORT.md`

### M2: Agent Context Address Space

目标：

- 在 Agent 进程用户地址空间中映射固定大小的 Agent Context 区。
- 实现 `sys_agent_create` 或等价创建/标记机制，初始化 PCB 字段并分配 Context 区。
- 让用户态 Agent 能直接读写自己的 Context 区。

交付：

- Agent Context 默认大小为 `64KB`。
- 内核记录 `agent_context_base` 和 `agent_context_size`。
- 测试覆盖 Context 区读写、越界保护、普通进程兼容性。
- `Docs/M2/DESIGN.md`
- `Docs/M2/IMPLEMENTATION_LOG.md`
- `Docs/M2/TEST_REPORT.md`

### M3: Structured Tool Call Interface

目标：

- 定义固定结构体或 TLV 风格 Tool Call 协议。
- 实现 `sys_tool_list` 和 `sys_tool_call`。
- 实现至少 3 个内核工具：
  - `get_system_status`
  - `query_process`
  - `send_message`

交付：

- 用户态 Agent 可调用 3 个工具并得到结构化响应。
- 错误 tool id、参数数量错误、参数类型错误都有明确错误码。
- 工具结果优先写入 Agent Context 区，响应中返回 offset/length。
- `Docs/M3/DESIGN.md`
- `Docs/M3/IMPLEMENTATION_LOG.md`
- `Docs/M3/TEST_REPORT.md`

### M4: Context Path Management

目标：

- 在 Agent Context 区实现 Context Path 环形缓冲区。
- 实现 `sys_context_push`、`sys_context_query`、`sys_context_rollback`、`sys_context_clear`。
- 内核维护路径元信息，用户态存储请求摘要和结果摘要。

交付：

- Agent 可连续记录至少 5 轮工具调用。
- rollback 和 clear 行为正确。
- 超过 quota 后执行 FIFO 淘汰。
- `Docs/M4/DESIGN.md`
- `Docs/M4/IMPLEMENTATION_LOG.md`
- `Docs/M4/TEST_REPORT.md`

### M5: Agent Loop Wakeup

目标：

- 实现 Agent 心跳机制。
- 实现 `sys_agent_heartbeat_set`、`sys_agent_heartbeat_stop`、`sys_agent_wait`。
- 让 `send_message` 可以唤醒等待中的目标 Agent。

交付：

- Agent 无事件时能 sleep/wait，不持续占用 CPU。
- 心跳到达后 Agent 被唤醒。
- 消息到达后目标 Agent 被唤醒。
- 多 Agent demo 稳定运行。
- `Docs/M5/DESIGN.md`
- `Docs/M5/IMPLEMENTATION_LOG.md`
- `Docs/M5/TEST_REPORT.md`

### M6: File Attribute Query

目标：

- 实现内存态文件属性表。
- 支持文件属性设置、查询、删除。
- 支持至少 `type`、`owner`、`tag`、`priority` 四类属性。
- 实现 `query_file` 工具。

交付：

- Agent 可通过属性查询文件，无需事先知道完整路径。
- 支持多条件 AND 查询。
- 提供遍历查询与属性查询的对比测试。
- `Docs/M6/DESIGN.md`
- `Docs/M6/IMPLEMENTATION_LOG.md`
- `Docs/M6/TEST_REPORT.md`

### M7: Integrated Demo

目标：

- 实现“Agent 系统管理员”综合演示。
- 整合 Agent 创建、Tool Call、Context Path、心跳/消息唤醒、文件属性查询。
- 提供可重复运行的用户态 demo 命令。

交付：

- `agent_demo basic`
- `agent_demo loop`
- `agent_demo fs_query_bench`
- `agent_demo full`
- demo 输出能清晰展示每轮 Agent Loop 和 Context Path。
- `Docs/M7/DESIGN.md`
- `Docs/M7/IMPLEMENTATION_LOG.md`
- `Docs/M7/TEST_REPORT.md`

### M8: Final Documentation And Evaluation

目标：

- 汇总系统设计、测试结果、性能评估和演示说明。
- 完善 README、设计文档、评估文档。
- 做最终构建、测试、QEMU demo 验证。

交付：

- `README.md`
- `Docs/DESIGN.md`
- `Docs/EVALUATION.md`
- `Docs/M8/DESIGN.md`
- `Docs/M8/IMPLEMENTATION_LOG.md`
- `Docs/M8/TEST_REPORT.md`
- 远程分支 `feature/agent-os` 与本地最终状态同步。

## Execution Protocol

Work milestone by milestone.

For each milestone Mx:
1. Implement the scoped feature.
2. Run build/tests/demo checks relevant to Mx.
3. Fix failures before moving on.
4. Update:
   - Docs/Mx/DESIGN.md
   - Docs/Mx/IMPLEMENTATION_LOG.md
   - Docs/Mx/TEST_REPORT.md
5. Commit with message: `Mx: <summary>`.
6. Push to remote branch `feature/agent-os`.

No PR is required. The remote repository is synchronized by milestone pushes.
