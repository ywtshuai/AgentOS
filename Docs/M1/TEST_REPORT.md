# M1 Test Report

## 计划验证项

- 普通进程仍可正常创建和等待子进程。
- 普通进程调用 `agent_info(-1, ...)` 返回 `-2`。
- `agent_create` 可以创建 Agent 子进程并返回 pid。
- 父进程可以查询直接 Agent 子进程的元数据。
- Agent 子进程可以查询自身元数据。
- Agent 子进程可以直接读写 64KB Agent Context 区域首尾地址。

## 新增测试程序

```text
agent_m1
agent_m1_child
```

`agent_m1` 会创建 `agent_m1_child`，校验 pid、agent type、heartbeat interval、resource quota 和 context size，然后等待子进程退出。

`agent_m1_child` 会查询自身 Agent 元数据，并对 Agent Context 的第一个字节和最后一个字节执行 volatile 写读校验。

## 已执行命令

```powershell
cargo fmt
.\scripts\check-env.ps1 -Strict
rg "Agent|agent_|SYSCALL_AGENT|AGENT_CONTEXT|TaskControlBlock" rcore/os/src rcore/user/src -n
```

## 结果

- `cargo fmt`：失败，当前环境找不到 `cargo`。
- `.\scripts\check-env.ps1 -Strict`：失败，缺少必要工具：
  - `rustup`
  - `rustc`
  - `cargo`
  - `make`
  - `qemu-system-riscv64`
- `rg ...`：通过，用于确认 M1 相关入口、常量、syscall 分发、用户态封装和测试程序均已落到预期文件。

## 未能执行的命令

由于本机缺少构建和仿真工具，以下命令暂未执行：

```powershell
cd rcore\os
make build
make run
```

装好工具链后建议运行：

```powershell
cd rcore\os
make build
make run
```

在 usershell 中运行：

```text
agent_m1
```

预期输出包括：

```text
agent_m1_child passed
agent_m1 passed
```
