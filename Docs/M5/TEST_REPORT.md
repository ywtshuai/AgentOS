# M5 Test Report

## 测试程序

```text
agent_m5
```

覆盖内容：

1. 普通进程调用 `agent_wait()` 返回 `-3`。
2. 普通进程调用 `agent_heartbeat_set()` / `agent_heartbeat_stop()` 返回 `-3`。
3. Agent 设置 30ms 心跳后调用 `agent_wait()`，心跳到达后被唤醒。
4. `agent_wait()` 返回值包含 `AGENT_WAKE_HEARTBEAT`。
5. 停止心跳后 `agent_info()` 显示 `heartbeat_interval == 0`。
6. Worker-Agent 调用 `agent_wait()` 后进入等待。
7. Admin-Agent 通过 `send_message` 唤醒 Worker-Agent。
8. Worker-Agent 的 `agent_wait()` 返回值包含 `AGENT_WAKE_MESSAGE`。
9. Worker-Agent 消费消息后 `pending_messages == 0`。

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
- `git diff --check`：通过。
- `make kernel`：通过。
- `make fs-img`：通过。
- QEMU 启动：通过。
- usershell 中运行 `agent_m5`：通过。
- usershell 中回归 `agent_m4`：通过。
- usershell 中回归 `agent_m3`：通过。
- usershell 中回归 `agent_m2`：通过。
- usershell 中回归 `agent_m1`：通过。

`agent_m5` 关键输出：

```text
agent_m5 non-agent guard passed
agent_m5 heartbeat wake passed
agent_m5 worker message wake passed
agent_m5 message wake passed
agent_m5 passed
Shell: Process 2 exited with code 0
```

回归输出：

```text
agent_m4 passed
Shell: Process 2 exited with code 0
agent_m3 passed
Shell: Process 2 exited with code 0
agent_m2 passed
Shell: Process 2 exited with code 0
agent_m1 passed
Shell: Process 2 exited with code 0
```

## 环境说明

与 M2-M4 相同，本次内核构建使用 stable 工具链配合 `RUSTC_BOOTSTRAP=1`。为避免 host 侧 `easy-fs-fuse` 继承内核 linker flags，仍采用“内核构建、手动 objcopy、文件系统构建、直接 QEMU 启动”的验证流程。
