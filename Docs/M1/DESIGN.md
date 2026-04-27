# M1 Design

## 目标

M1 只实现 Agent 进程元数据的内核表示和查询接口，不负责 Agent Context 地址空间映射，也不负责 Agent 创建机制。这些内容推迟到 M2。

本里程碑交付：

- 在 PCB / task control block 中保存 Agent 元数据。
- 普通进程默认不是 Agent。
- `sys_agent_info` 可以查询当前进程或指定 pid 的 Agent 元数据。
- 用户态 M1 测试程序验证普通进程与 Agent 元数据字段互不影响。

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

M1 中 `agent_context_base` 和 `agent_context_size` 作为元数据字段保留，但默认值为 `0`。M2 会在 Agent 创建和地址空间映射完成后填充实际地址和大小。

## Loop 状态

`AgentLoopState` 目前只作为元数据枚举存在：

- `Ready`
- `Waiting`
- `Running`
- `Finished`

M1 默认初始化为 `Ready`。实际等待、心跳和唤醒语义在 M5 实现。

## 系统调用

新增 syscall id：

- `501`: `sys_agent_info(pid, info_ptr)`

查询规则：

- `pid == -1`：查询当前进程。
- `pid >= 0`：从 `INITPROC` 进程树递归查找指定 pid。

返回值：

- `0`：查询成功，`info_ptr` 被写入 Agent 元数据。
- `-1`：目标 pid 不存在。
- `-2`：目标 pid 存在，但目标进程不是 Agent。

## 用户态接口

`user_lib` 新增：

- `AgentInfo`
- `agent_info(pid, &mut AgentInfo)`

M1 不提供 `agent_create`。Agent 创建、Context 映射和 Context 读写测试属于 M2。

## 测试程序

新增：

- `agent_m1`

测试内容：

- 当前普通进程调用 `agent_info(-1, ...)` 返回 `-2`。
- `fork` 创建的普通子进程调用 `agent_info(-1, ...)` 返回 `-2`。
- 父进程按子进程 pid 调用 `agent_info(pid, ...)` 返回 `-2`。
