# M0 Design

## 本里程碑目标

M0 的目标是建立可继续迭代的 Agent-OS 工程基线：

- 在 `feature/agent-os` 分支上导入 rCore-Tutorial v3 基础代码。
- 选择支持进程、系统调用、虚拟内存和简单文件系统的 rCore `ch6` 作为后续扩展起点。
- 增加可重复的本地环境检查入口，提前发现 Rust、QEMU、Make 等缺失项。

## 关键设计

- rCore 基线代码放在 `rcore/` 子目录，避免覆盖已有 `Docs/` 文档。
- M1 以后以内核 `rcore/os/src` 和用户程序 `rcore/user/src` 为主要实现位置。
- 当前不接入真实外部 LLM API；后续用户态 Agent demo 使用 deterministic mock policy。
- `ch6` 已包含 simple file system，相比更早章节更适合后续 M5 文件属性查询扩展。

## 接口/系统调用变化

M0 不新增系统调用。计划从 M1 开始增加：

- `sys_agent_create`
- `sys_agent_info`

## 与比赛要求的对应关系

- 满足“基于教学操作系统内核”的起步要求。
- 满足“RISC-V 64 / QEMU”目标平台准备。
- 为基础任务一到三提供可扩展的 rCore 代码基线。
