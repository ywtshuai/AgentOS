# M3 Implementation Log

## 修改内容

- 新增 syscall id：
  - `502`: `sys_tool_call`
  - `503`: `sys_tool_list`
- 新增固定二进制 Tool Call 协议：
  - `ToolRequest`
  - `ToolParam`
  - `ToolResponse`
  - `ToolInfo`
- 实现三个内核工具：
  - `get_system_status`
  - `query_process`
  - `send_message`
- `sys_tool_call()` 增加 Agent 权限检查，普通进程调用返回 `-3`。
- 工具结果写入 Agent Context，并通过 `result_offset + result_len` 返回给用户态。
- 新增用户态封装：
  - `tool_call()`
  - `tool_list()`
  - 对应协议结构体和常量。
- 新增用户态测试程序 `agent_m3`。
- 修正 Agent 父进程 `fork()` 语义：
  - 子进程默认仍不是 Agent。
  - 若父进程是 Agent，子进程移除复制来的 Agent Context 映射，避免后续 `agent_create()` 冲突。

## 调试记录

第一次 QEMU 运行中，`agent_m3` 的 worker 子进程调用 `agent_create()` 返回 `-2`。原因是 `fork()` 复制了父 Agent 的 Agent Context 页表映射，但子进程 PCB 的 `agent` 字段为 `None`。修复为在 `TaskControlBlock::fork()` 中，若父进程是 Agent，则对子进程 `MemorySet` 调用 `remove_area_with_start_vpn(AGENT_CONTEXT_BASE)`。

第二次 QEMU 运行仍看到旧行为。原因是 `make kernel` 只更新 ELF，不刷新 `os.bin`。手动执行：

```bash
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```

后重新启动 QEMU，`agent_m3` 通过。

## 关键文件

- `rcore/os/src/syscall/mod.rs`
- `rcore/os/src/syscall/process.rs`
- `rcore/os/src/task/mod.rs`
- `rcore/os/src/task/task.rs`
- `rcore/user/src/syscall.rs`
- `rcore/user/src/lib.rs`
- `rcore/user/src/bin/agent_m3.rs`

## 已知限制

- `send_message` 当前只完成结构化校验和结果返回，尚未实现消息队列、阻塞等待和唤醒；这些放到 Agent Loop 里程碑。
- M3 的 Context 写入只是工具结果缓存，还不是正式 Context Path 节点布局。
- `query_process` 当前基于 INITPROC 进程树递归遍历，尚未使用索引。
