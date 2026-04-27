# M2 Design

## 目标

M2 实现 Agent Context 地址空间支持：

- 新增 `sys_agent_create(agent_type, heartbeat_interval, resource_quota)`。
- 将当前进程登记为 Agent，并初始化 M1 已定义的 PCB 元数据。
- 在用户地址空间固定映射 64KB Agent Context 区。
- 用户态 Agent 可直接通过返回的 `agent_context_base` 读写 Context。

## 地址空间布局

Agent Context 使用固定高地址区间：

```text
AGENT_CONTEXT_BASE = TRAP_CONTEXT - 64KB
AGENT_CONTEXT_SIZE = 64KB
```

该区域位于 trap context 下方，避开普通 ELF 段、用户栈和 trampoline。映射权限为：

```text
R | W | U
```

内核只在 PCB 中保存 base/size 等元数据；Context 内容由用户态直接读写，后续 M3/M4 会把工具结果和 Context Path 写入该区域。

## 创建语义

M2 采用“当前进程登记”为 Agent 的等价创建机制：

```text
sys_agent_create(agent_type, heartbeat_interval, resource_quota)
```

返回值：

- `0`：创建成功。
- `-1`：当前进程已经是 Agent。
- `-2`：固定 Context 区间已被有效映射，无法创建。

登记成功后，`sys_agent_info(-1, &mut info)` 会返回 Agent 类型、心跳周期、资源配额、loop state、Context base 和 Context size。

## 兼容性

- 普通进程默认仍为非 Agent。
- `fork()` 创建的子进程默认不是 Agent，保持 M1 行为。
- Agent 进程执行 `exec()` 时会重新映射 Agent Context，避免元数据指向不存在的用户地址。

## 关键实现点

- `MemorySet::insert_framed_area_checked()` 只把 valid PTE 视为冲突，避免页表中空 PTE 槽位造成误判。
- `sys_agent_create()` 不在内核态调用 `memory_set.activate()`；rCore 的 trap return 会切换回用户页表并刷新地址空间。
- 补齐 `rcore/os/src/entry.asm`，修复基线缺失导致内核无法链接的问题。
