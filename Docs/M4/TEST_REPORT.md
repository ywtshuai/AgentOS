# M4 Test Report

## 测试程序

```text
agent_m4
```

覆盖内容：

1. 普通进程调用 `context_query()` 返回 `-3`。
2. Agent 连续 `context_push()` 5 个节点。
3. 每个节点的 `prev_id` 串联成路径。
4. 用户态直接从 Agent Context offset 读取 request/result 摘要。
5. `context_query()` 返回节点数、active 节点和节点列表。
6. `context_rollback()` 回退到第 3 个节点，并截断后续 live 节点。
7. 大节点触发 quota wrap，旧节点被 FIFO 淘汰。
8. `context_clear()` 清空路径。

## 已执行命令

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
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
- usershell 中运行 `agent_m4`：通过。
- usershell 中回归 `agent_m3`：通过。
- usershell 中回归 `agent_m2`：通过。
- usershell 中回归 `agent_m1`：通过。

`agent_m4` 关键输出：

```text
agent_m4 non-agent guard passed
agent_m4 push/query passed
agent_m4 rollback passed
agent_m4 fifo wrap passed
agent_m4 clear passed
agent_m4 passed
Shell: Process 2 exited with code 0
```

回归输出：

```text
agent_m3 passed
Shell: Process 2 exited with code 0
agent_m2 passed
Shell: Process 2 exited with code 0
agent_m1 passed
Shell: Process 2 exited with code 0
```

## 环境说明

与 M2/M3 相同，本次内核构建使用 stable 工具链配合 `RUSTC_BOOTSTRAP=1`。为避免 host 侧 `easy-fs-fuse` 继承内核 linker flags，仍采用“内核构建、手动 objcopy、文件系统构建、直接 QEMU 启动”的验证流程。
