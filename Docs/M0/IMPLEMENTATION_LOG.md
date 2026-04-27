# M0 Implementation Log

## 修改了什么

- 导入 `rCore-Tutorial-v3` 的 `ch6` 分支源码到 `rcore/`。
- 新增顶层 `README.md`，说明目录结构、环境检查和基础构建命令。
- 新增 `scripts/check-env.ps1`，检查 Git、Rust、Cargo、Make 和 QEMU。
- 新增 `.gitignore`，过滤常见构建产物。
- 新增 M0 里程碑文档目录。

## 为什么这样改

当前仓库只有文档，没有 OS 代码。M0 先建立最小可演进基线，后续 M1 可以直接在 rCore 的进程控制块、地址空间和 syscall 分发路径上实现 Agent 进程模型。

选择 `ch6` 是因为它已经具备进程、文件系统和用户程序构建流程，能覆盖后续 Agent Context、Tool Call、Context Path 和文件属性查询的共同基础。

## 关键文件

- `rcore/os/src/task/task.rs`
- `rcore/os/src/mm/memory_set.rs`
- `rcore/os/src/syscall/mod.rs`
- `rcore/user/src/syscall.rs`
- `scripts/check-env.ps1`
- `README.md`

## 已知限制

- 本机当前缺少 Rust、Cargo、Make、QEMU，暂时无法完成 QEMU 启动验证。
- M0 尚未修改内核行为，不包含 Agent 专属 PCB 字段或 syscall。
- rCore 上游代码以子目录方式导入，后续改动会在 `rcore/` 内直接演进。
