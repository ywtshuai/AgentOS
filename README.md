# Agent-OS

Agent-OS 是基于 rCore-Tutorial v3 的教学操作系统扩展项目，面向 AI Agent OS 竞赛实现。项目把 AI Agent 作为操作系统的一等进程来支持，在内核中加入 Agent Context 内存区、结构化工具调用、Context Path 管理、心跳/消息唤醒以及按属性查询文件等机制。

系统运行目标是 RISC-V 64 QEMU。用户态 Agent 使用确定性的 demo 程序模拟 Agent Loop，不接入外部 LLM API。

## 仓库结构

- `rcore/`：基于 rCore-Tutorial v3 的内核、用户态库和 demo 程序。
- `Docs/PLAN.md`：里程碑计划和执行协议。
- `Docs/CompetitionRequirements.md`：竞赛原始任务说明。
- `Docs/M0` 到 `Docs/M8`：每个里程碑的设计、实现日志和测试报告。
- `Docs/design.md`：Agent-OS 总体架构设计。
- `Docs/evaluation.md`：功能测试和性能评估。
- `scripts/check-env.ps1`：Windows PowerShell 环境检查脚本。

## 环境要求

需要安装：

- Rust 工具链，包括 `rustup`、`cargo` 和 `rustc`。
- `riscv64gc-unknown-none-elf` Rust target。
- `cargo-binutils`、`rust-src`、`llvm-tools-preview`。
- `make`。
- `qemu-system-riscv64`。

Windows PowerShell 下可运行：

```powershell
.\scripts\check-env.ps1
```

当前仓库验证过的构建方式使用 stable Rust 配合 `RUSTC_BOOTSTRAP=1`，用于兼容上游 rCore 代码中的 nightly feature。

## 编译

```bash
cd rcore/os
cargo +stable fmt
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
```

## 运行

```bash
cd rcore/os
make run
```

进入 usershell 后可运行：

```text
agent_m1
agent_m2
agent_m3
agent_m4
agent_m5
agent_m6
agent_demo basic
agent_demo loop
agent_demo fs_query_bench
agent_demo full
```

`agent_demo full` 是主要综合演示场景：Admin-Agent 通过心跳唤醒，查询系统状态，按属性查询文件，向 Worker-Agent 发送消息，并打印完整 Context Path。

## Agent-OS 系统调用

| ID | 系统调用 | 功能 |
| --- | --- | --- |
| 500 | `sys_agent_create` | 将当前进程登记为 Agent，并映射 64KB Agent Context 区。 |
| 501 | `sys_agent_info` | 查询当前进程或指定 pid 的 Agent 元信息。 |
| 502 | `sys_tool_call` | 执行结构化内核工具请求，并把结果写入 Agent Context。 |
| 503 | `sys_tool_list` | 查询当前可用的内核工具列表。 |
| 504 | `sys_context_push` | 追加一个 Context Path 节点。 |
| 505 | `sys_context_query` | 查询近期 Context Path 节点。 |
| 506 | `sys_context_rollback` | 回滚 active Context Path 节点。 |
| 507 | `sys_context_clear` | 清空当前 Context Path。 |
| 508 | `sys_agent_heartbeat_set` | 配置 Agent Loop 心跳唤醒。 |
| 509 | `sys_agent_heartbeat_stop` | 停止心跳唤醒。 |
| 510 | `sys_agent_wait` | 挂起 Agent，直到心跳或消息事件唤醒。 |
| 511 | `sys_file_attr_set` | 为文件绑定可查询属性。 |
| 512 | `sys_file_attr_delete` | 删除文件属性项。 |

## 内核工具

| 工具 | 功能 |
| --- | --- |
| `get_system_status` | 返回进程、Agent、内存和时间摘要。 |
| `query_process` | 按进程状态和 Agent 类型查询进程。 |
| `send_message` | 发送结构化 Agent 消息，并唤醒目标 Agent。 |
| `query_file` | 按 `type`、`owner`、`tag`、`priority` 属性查询文件。 |

## 文档入口

建议先阅读 [Docs/design.md](Docs/design.md) 了解总体架构，再阅读 [Docs/evaluation.md](Docs/evaluation.md) 查看测试结果。每个里程碑在 `Docs/Mx/` 下也保留了对应的 `DESIGN.md`、`IMPLEMENTATION_LOG.md` 和 `TEST_REPORT.md`。
