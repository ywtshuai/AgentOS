# M1 Design

## 目标

M1 实现 Agent 进程与地址空间的最小闭环：

- 普通进程可以通过 `sys_agent_create` 创建 Agent 子进程。
- Agent 进程拥有独立的 PCB 元数据。
- Agent 地址空间映射固定的 64KB Agent Context 区域。
- Agent 可以读写自己的 Agent Context。
- 内核可以通过 `sys_agent_info` 返回 Agent 元数据。
- 普通进程调用当前进程的 Agent 专属查询时返回权限错误。

## 地址空间布局

新增常量：

- `AGENT_CONTEXT_BASE = 0x1000_0000`
- `AGENT_CONTEXT_SIZE = 64KB`

Agent Context 使用 framed user mapping，权限为 `R | W | U`。它位于常规 ELF 段和用户栈之上，与 trampoline/trap context 的高地址区域分离。

## PCB 元数据

`TaskControlBlockInner` 新增：

- `agent: Option<AgentMeta>`

`None` 表示普通进程，`Some` 表示 Agent 进程。`AgentMeta` 保存：

- `agent_type`
- `heartbeat_interval`
- `resource_quota`
- `loop_state`
- `context_path_meta`
- `agent_context_base`
- `agent_context_size`

M1 中 `loop_state` 初始化为 `Ready`，后续 M4 会扩展等待与唤醒语义。

## 系统调用

新增 syscall id：

- `500`: `sys_agent_create(args_ptr)`
- `501`: `sys_agent_info(pid, info_ptr)`

`sys_agent_create` 通过用户态传入的 `AgentCreateArgs` 读取可执行文件路径和初始 Agent 参数，创建一个 Agent 子进程并加入调度队列。

`sys_agent_info` 支持查询当前进程或直接子进程：

- 返回 `0`：查询成功。
- 返回 `-1`：目标进程不存在或不是当前进程的直接子进程。
- 返回 `-2`：目标进程存在，但不是 Agent。

## 用户态接口

`user_lib` 新增：

- `agent_create(path, agent_type, heartbeat_interval, resource_quota)`
- `agent_info(pid, &mut AgentInfo)`

新增测试程序：

- `agent_m1`: 普通进程创建 Agent 子进程并查询元数据。
- `agent_m1_child`: Agent 子进程查询自身元数据，并读写 Agent Context 首尾字节。
