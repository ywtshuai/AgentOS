# M3 Test Report

## 测试程序

```text
agent_m3
```

覆盖内容：

1. `tool_list()` 返回 3 个工具。
2. 普通进程调用 `tool_call()` 返回 `-3`。
3. Agent 调用 `get_system_status`，结果写入 Agent Context。
4. Agent 调用 `query_process(agent_type=ADMIN_AGENT)`，返回结构化进程列表。
5. Agent fork 出普通子进程，子进程可重新 `agent_create()` 成 Worker-Agent。
6. Admin-Agent 调用 `send_message(target_pid=worker_pid)`，返回结构化确认结果。
7. 未知 tool id 返回 `-4`。

## 已执行命令

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
qemu-system-riscv64 ...
```

## 结果

- `cargo +stable fmt`：通过。
- 用户态 `RUSTC_BOOTSTRAP=1 cargo +stable check`：通过。
- 内核 `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check`：通过。
- `make kernel`：通过。
- `make fs-img`：通过。
- QEMU 启动：通过。
- usershell 中运行 `agent_m3`：通过。
- usershell 中回归 `agent_m2`：通过。
- usershell 中回归 `agent_m1`：通过。

`agent_m3` 关键输出：

```text
agent_m3 tool_list passed
agent_m3 non-agent guard passed
agent_m3 get_system_status passed
agent_m3 query_process passed
agent_m3 send_message passed
agent_m3 bad_tool passed
agent_m3 passed
Shell: Process 2 exited with code 0
```

回归输出：

```text
agent_m2 passed
Shell: Process 2 exited with code 0
agent_m1 passed
Shell: Process 2 exited with code 0
```

## 环境说明

与 M2 相同，内核构建使用 stable 工具链配合 `RUSTC_BOOTSTRAP=1`。`make run` 不能直接带内核 `RUSTFLAGS`，否则 host 侧 `easy-fs-fuse` 会继承 `-Tsrc/linker.ld` 并链接失败。因此本次仍采用“内核构建、手动 objcopy、文件系统构建、直接 QEMU 启动”的验证流程。
