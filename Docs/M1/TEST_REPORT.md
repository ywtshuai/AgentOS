# M1 Test Report

## 计划验证项

- 普通进程默认不是 Agent。
- 普通子进程默认不是 Agent。
- `sys_agent_info(-1, ...)` 可以查询当前进程，并在当前进程不是 Agent 时返回 `-2`。
- `sys_agent_info(pid, ...)` 可以查找指定 pid，并在目标进程不是 Agent 时返回 `-2`。
- M1 不验证 Agent Context 读写，相关测试移到 M2。

## 测试程序

```text
agent_m1
```

`agent_m1` 执行流程：

1. 当前进程调用 `agent_info(-1, ...)`，期望返回 `-2`。
2. `fork()` 创建普通子进程。
3. 子进程调用 `agent_info(-1, ...)`，期望返回 `-2`。
4. 父进程调用 `agent_info(child_pid, ...)`，期望返回 `-2`。
5. 父进程等待子进程退出，期望退出码为 `0`。

## 已执行命令

```powershell
cargo fmt
.\scripts\check-env.ps1 -Strict
git diff --check
rg "agent_create|sys_agent_create|AGENT_CONTEXT_BASE|AGENT_CONTEXT_SIZE|map_agent_context" rcore/os/src rcore/user/src -n
```

## 结果

- `cargo fmt`：失败，当前环境找不到 `cargo`。
- `.\scripts\check-env.ps1 -Strict`：失败，缺少必要工具：
  - `rustup`
  - `rustc`
  - `cargo`
  - `make`
  - `qemu-system-riscv64`
- `git diff --check`：通过，未发现空白错误。
- `rg ...`：无匹配，退出码为 `1`，表示代码中不再包含 Agent 创建和 Context 映射入口。

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
agent_m1 child passed
agent_m1 passed
```
