# M2 Test Report

## 测试程序

```text
agent_m2
```

覆盖内容：

1. 普通进程调用 `agent_info(-1, ...)` 返回 `-2`。
2. `agent_create(7, 250, 4096)` 返回 `0`。
3. `agent_info(-1, ...)` 返回 Agent 元数据。
4. `agent_context_size == 65536`。
5. 用户态直接写入并读取 Context 首字节和末字节。
6. 重复 `agent_create()` 返回 `-1`。
7. `fork()` 子进程默认不是 Agent。

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
- usershell 中运行 `agent_m2`：通过。
- usershell 中回归 `agent_m1`：通过。

`agent_m2` 关键输出：

```text
agent_m2 create passed
agent_m2 context base=0xfffffffffffee000 size=65536
agent_m2 context rw passed
agent_m2 duplicate create passed
agent_m2 child passed
agent_m2 passed
Shell: Process 2 exited with code 0
```

`agent_m1` 回归输出：

```text
agent_m1 child passed
agent_m1 passed
Shell: Process 2 exited with code 0
```

## 环境说明

当前环境的 `nightly-2025-02-18` rustup 工具链处于部分安装/冲突状态，直接 `cargo` 会反复尝试修复 nightly 并失败。因此本次验证使用系统已安装的 stable 工具链配合：

```bash
RUSTC_BOOTSTRAP=1
RUSTFLAGS='-Afunction-casts-as-integer ...'
```

该 `RUSTFLAGS` 只用于内核构建，不能传给 host 侧 `easy-fs-fuse`，否则 host 构建会错误继承 `-Tsrc/linker.ld`。
