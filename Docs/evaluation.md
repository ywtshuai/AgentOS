# Agent-OS 评估报告

## 环境

本项目在 rCore-Tutorial v3 构建流程下，通过 RISC-V 64 QEMU 进行评估。已验证命令使用 stable Rust，并配合 `RUSTC_BOOTSTRAP=1`。

## 构建检查

已执行检查：

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
git diff --check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
```

结果：M7 验证记录中，上述构建检查全部通过。

## 功能测试

回归程序：

| 程序 | 覆盖内容 | 结果 |
| --- | --- | --- |
| `agent_m1` | 普通进程默认不是 Agent；`agent_info` 错误路径 | 通过 |
| `agent_m2` | Agent 创建、Agent Context 映射、Context 直接读写 | 通过 |
| `agent_m3` | 工具列表、系统状态、进程查询、消息工具、工具错误处理 | 通过 |
| `agent_m4` | Context Path push、query、rollback、clear、FIFO 配额行为 | 通过 |
| `agent_m5` | 心跳等待、消息唤醒、Blocked Agent 调度 | 通过 |
| `agent_m6` | 文件属性、多条件文件查询、属性删除行为 | 通过 |

综合 demo 程序：

| 程序 | 覆盖内容 | 结果 |
| --- | --- | --- |
| `agent_demo basic` | Agent 创建、工具调用、结果读取、Context Path 写入 | 通过 |
| `agent_demo loop` | 心跳唤醒、Worker-Agent 消息唤醒、Context Path | 通过 |
| `agent_demo fs_query_bench` | 文件属性和查询访问次数对比 | 通过 |
| `agent_demo full` | 串联 M1-M6 机制的 Admin-Agent 端到端场景 | 通过 |

M7 QEMU 关键输出：

```text
agent_demo basic: passed
agent_demo loop: passed
agent_demo fs_query_bench: passed
agent_demo full: passed
agent_m6 passed
agent_m5 passed
agent_m4 passed
agent_m3 passed
agent_m2 passed
agent_m1 passed
```

自动化 QEMU 命令在 usershell 回到等待输入状态后由 `timeout` 结束。因此，当所有 passed 输出已经打印后，外层命令退出码为 `124` 是该测试脚本下的预期现象。

## 文件查询性能

M6/M7 benchmark 创建 4 个文件并绑定属性。查询条件为：

```text
type=memory AND owner=worker AND tag=social
```

返回 1 个匹配文件：

```text
query_file matches=1 traversal=4 indexed=2 first=m7_b
full file_query matches=1 traversal=4 indexed=2
```

解释：

- 全量遍历需要检查 4 个属性项。
- 简化索引模型根据第一个查询条件选出候选项，只需检查 2 个条目。
- 在 demo 数据集上，候选查询访问次数减少 50%。

## Agent Context 访问

Agent Context 映射在用户态地址空间中。系统调用把工具结果和 Context Path 摘要写入该区域后，Agent 可以直接读取，不需要为每次结果读取再发起一次系统调用。这体现了赛题要求的机制与策略分离：

- 内核负责 Agent 身份、配额和元数据更新。
- 用户态策略直接读取并解释 Context 缓存字节。

## 稳定性说明

M7 记录的综合场景反复覆盖 Agent 创建、唤醒、文件查询、Context Path 和 shell 命令映射，没有出现 kernel panic。当前已知限制记录在各里程碑报告中，主要包括：

- 文件属性是内存态，重启后不保留。
- Context Path FIFO 在写入游标换轮时粒度较粗。
- 心跳扫描按进程树线性遍历，适合当前教学 demo 规模。
