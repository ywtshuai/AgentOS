# M1 Implementation Log

## 修改内容

- 在 `TaskControlBlockInner` 增加 `agent: Option<AgentMeta>`。
- 增加 `AgentMeta`，保存 Agent 类型、心跳间隔、资源配额、loop 状态、context 元数据字段。
- 增加 `AgentLoopState`，为后续 Agent Loop 里程碑预留状态枚举。
- 在普通进程创建路径中将 `agent` 初始化为 `None`。
- 调整 `fork()`：普通 fork 创建的子进程默认不是 Agent，避免隐式继承 Agent 身份。
- 增加 `sys_agent_info`，支持查询当前进程或指定 pid 的 Agent 元数据。
- `sys_agent_info` 从 `INITPROC` 进程树递归查找 pid，目标存在但不是 Agent 时返回 `-2`。
- 增加用户态 `AgentInfo` 和 `agent_info()` 封装。
- 调整 M1 测试程序 `agent_m1`，只验证普通父子进程默认非 Agent。

## 从 M1 移出的内容

根据补充后的 `Docs/PLAN.md`，以下内容不再属于 M1，已从 M1-fix 中移出，后续放到 M2：

- `AGENT_CONTEXT_BASE`
- `AGENT_CONTEXT_SIZE`
- Agent Context 地址空间映射
- `sys_agent_create`
- 用户态 `agent_create`
- Agent Context 直接读写测试

## 关键文件

- `rcore/os/src/task/task.rs`
- `rcore/os/src/task/mod.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m1.rs`

## 行为说明

M1 中普通进程默认不是 Agent：

```text
agent_info(-1, &mut info) == -2
```

父进程 fork 出的普通子进程同样不是 Agent：

```text
agent_info(child_pid, &mut info) == -2
```

`sys_agent_info(pid, info_ptr)` 的 pid 查询范围是从 `INITPROC` 开始的进程树。M1 目前没有公开 Agent 创建入口，因此正向 Agent 查询路径会在 M2 引入 `sys_agent_create` 后补充测试。

## 已知限制

- M1 只建立元数据结构和查询接口，尚未提供将普通进程标记为 Agent 的用户态创建路径。
- Context 地址、Context 大小字段在 M1 中保留但默认是 `0`，M2 负责初始化真实值。
- 当前本机缺少 Rust/Cargo/Make/QEMU，无法执行真实编译和 QEMU demo。
