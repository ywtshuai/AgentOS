# M6 Test Report

## 测试程序

```text
agent_m6
```

覆盖内容：

1. 创建 4 个测试文件。
2. 为文件设置 `type`、`owner`、`tag`、`priority` 属性。
3. Agent 调用 `query_file(type=memory AND owner=agent-b AND tag=social)`。
4. 查询结果写入 Agent Context，并由用户态直接读取。
5. 查询结果只返回目标文件 `m6_b`。
6. 输出遍历查询与属性索引查询访问次数对比。
7. 删除 `m6_b` 属性后再次查询，确认结果为空。
8. 回归 M1-M5 测试程序。

## 已执行命令

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
git diff --check
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
- usershell 中运行 `agent_m6`：通过。
- usershell 中回归 `agent_m5`、`agent_m4`、`agent_m3`、`agent_m2`、`agent_m1`：通过。

`agent_m6` 关键输出：

```text
agent_m6 file attrs set passed
agent_m6 query_file passed traversal=4 indexed=2
agent_m6 delete passed
agent_m6 passed
Shell: Process 2 exited with code 0
```

回归输出：

```text
agent_m5 passed
Shell: Process 2 exited with code 0
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

与 M2-M5 相同，本次内核构建使用 stable 工具链配合 `RUSTC_BOOTSTRAP=1`。为避免 host 侧 `easy-fs-fuse` 继承内核 linker flags，仍采用“内核构建、手动 objcopy、文件系统构建、直接 QEMU 启动”的验证流程。
