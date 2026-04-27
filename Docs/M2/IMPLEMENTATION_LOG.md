# M2 Implementation Log

## 修改内容

- 新增 Agent Context 常量：
  - `AGENT_CONTEXT_BASE`
  - `AGENT_CONTEXT_SIZE`
- 新增 `MemorySet::insert_framed_area_checked()`，用于安全映射固定用户态 Context 区。
- 新增 syscall id `500`：`sys_agent_create`。
- 实现 `sys_agent_create()`：
  - 检查当前进程是否已是 Agent。
  - 映射 64KB 用户态 Agent Context。
  - 初始化 `AgentMeta`，写入 Agent 类型、心跳周期、资源配额和 Context base/size。
- 更新 `AgentMeta::new()`，M2 起默认填充真实 Context base/size。
- 更新 `exec()` 路径：Agent 进程 exec 后重新映射 Context 区，避免元数据失效。
- 新增用户态封装：
  - `agent_create()`
  - `AGENT_CONTEXT_SIZE`
- 新增用户态测试程序 `agent_m2`。
- 补齐 `rcore/os/src/entry.asm`，让内核具备 `_start` 入口和启动栈。

## 调试记录

第一次 QEMU 运行中，`agent_create()` 返回 `-2`。原因是映射冲突检查将无效 PTE 也视为已映射；修复为只在 `pte.is_valid()` 时判定冲突。

第二次 QEMU 运行中，`agent_create()` 卡住。原因是 syscall 中调用了 `memory_set.activate()`，导致内核态继续在用户页表中运行。修复为删除该调用，依赖 trap return 切换到用户页表。

## 关键文件

- `rcore/os/src/config.rs`
- `rcore/os/src/mm/memory_set.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/os/src/task/task.rs`
- `rcore/os/src/entry.asm`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m2.rs`

## 已知限制

- M2 只提供 Context 原始字节区，尚未定义内部布局。
- Context Path 环形缓冲区、工具结果写入 offset 和配额淘汰在 M3/M4 实现。
- `fork()` 后子进程默认不是 Agent；当前实现保持 M1 语义，不把 Agent 身份隐式继承给子进程。
