# M7 Test Report

## 测试程序

```text
agent_demo basic
agent_demo loop
agent_demo fs_query_bench
agent_demo full
```

回归程序：

```text
agent_m6
agent_m5
agent_m4
agent_m3
agent_m2
agent_m1
```

## 覆盖内容

- `basic` 覆盖 Agent 创建、Tool Call、Agent Context 结果读取、Context Path 写入和查询。
- `loop` 覆盖心跳等待、消息唤醒、多 Agent 协作和 Context Path。
- `fs_query_bench` 覆盖文件属性设置、多条件查询和访问次数对比。
- `full` 覆盖 M1-M6 的综合串联演示。
- M1-M6 回归确认已有里程碑没有被 M7 demo 和 shell 映射破坏。

## 已执行命令

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
git diff --check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
timeout 90s bash -lc "printf 'agent_demo basic\nagent_demo loop\nagent_demo fs_query_bench\nagent_demo full\nagent_m6\nagent_m5\nagent_m4\nagent_m3\nagent_m2\nagent_m1\n' | qemu-system-riscv64 ..."
```

## 结果

- `cargo +stable fmt`：通过。
- 用户态 `RUSTC_BOOTSTRAP=1 cargo +stable check`：通过。
- 内核 `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check`：通过。
- `git diff --check`：通过。
- `make kernel`：通过。
- `make fs-img`：通过。
- QEMU 启动：通过。
- `agent_demo basic`：通过。
- `agent_demo loop`：通过。
- `agent_demo fs_query_bench`：通过。
- `agent_demo full`：通过。
- `agent_m6` 到 `agent_m1` 回归：通过。

关键输出：

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

性能对比输出：

```text
query_file matches=1 traversal=4 indexed=2 first=m7_b
full file_query matches=1 traversal=4 indexed=2
```

Context Path 输出示例：

```text
context_path nodes=4 active=4
  node=1 prev=0 tool=1
  node=2 prev=1 tool=1
  node=3 prev=2 tool=4
  node=4 prev=3 tool=3
```

## 环境说明

与 M2-M6 相同，本次内核构建使用 stable 工具链配合 `RUSTC_BOOTSTRAP=1`。QEMU 自动化命令在所有程序通过后由 `timeout` 结束，因此最终 shell 命令退出码为 `124`，这是 usershell 持续等待输入导致的预期现象。
