# M1 Implementation Log

## 修改内容

- 在 `rcore/os/src/config.rs` 增加 Agent Context 固定地址和 64KB 大小常量。
- 在 `MemorySet` 增加 `map_agent_context()`，为 Agent 地址空间映射用户可读写的 context 区域。
- 在 `TaskControlBlockInner` 增加 `agent: Option<AgentMeta>`。
- 增加 `AgentMeta` 与 `AgentLoopState`，保存 Agent 类型、心跳间隔、资源配额、context 元数据和 context 地址范围。
- 增加 `TaskControlBlock::new_agent()`，从 ELF 创建 Agent 子进程并挂到当前进程 children。
- 扩展 `exec()`：如果当前进程已经是 Agent，exec 后保留 Agent 元数据并重新映射 Agent Context。
- 增加内核 syscall：
  - `sys_agent_create`
  - `sys_agent_info`
- 增加用户态封装：
  - `agent_create`
  - `agent_info`
  - `AgentCreateArgs`
  - `AgentInfo`
- 增加 M1 测试程序：
  - `rcore/user/src/bin/agent_m1.rs`
  - `rcore/user/src/bin/agent_m1_child.rs`

## 关键文件

- `rcore/os/src/config.rs`
- `rcore/os/src/mm/memory_set.rs`
- `rcore/os/src/task/task.rs`
- `rcore/os/src/task/mod.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m1.rs`
- `rcore/user/src/bin/agent_m1_child.rs`

## 行为说明

普通进程调用 `agent_info(-1, ...)` 会返回 `-2`，表示当前进程不是 Agent。

普通进程调用 `agent_create("agent_m1_child\0", 7, 100, 64 * 1024)` 后，内核会：

1. 从 easy-fs 根目录读取 `agent_m1_child` ELF。
2. 创建新的 TCB 和用户地址空间。
3. 映射固定 Agent Context 区域。
4. 初始化 Agent 元数据。
5. 将新任务加入当前进程 children 和调度队列。

Agent 子进程可以通过 `agent_info(-1, ...)` 查询自身元数据，并直接读写 `agent_context_base` 到 `agent_context_base + agent_context_size` 范围内的内存。

## 已知限制

- M1 只提供 Agent 创建和元数据查询，尚未实现 Tool Call、Context Path、Agent Loop 唤醒。
- `sys_agent_info` 当前只支持当前进程和直接子进程查询，避免在 M1 引入全局进程表遍历。
- `resource_quota` 在 M1 中作为元数据保存，具体淘汰和配额执行会在 M3 实现。
- 当前本机缺少 Rust/Cargo/Make/QEMU，无法执行真实编译和 QEMU demo。
