# M7 Implementation Log

## 修改内容

- 新增综合 demo 共享实现 `rcore/user/src/agent_demo_shared.rs`。
- 新增四个 demo 入口：
  - `agent_demo_basic`
  - `agent_demo_loop`
  - `agent_demo_fs_query_bench`
  - `agent_demo_full`
- 更新 `user_shell`，支持计划中的命令形式：
  - `agent_demo basic`
  - `agent_demo loop`
  - `agent_demo fs_query_bench`
  - `agent_demo full`
- demo 复用 M1-M6 已实现的用户态接口：
  - `agent_create`
  - `tool_call`
  - `context_push` / `context_query`
  - `agent_heartbeat_set` / `agent_wait`
  - `file_attr_set`

## Demo 行为

`agent_demo basic`：

- 创建 Admin-Agent。
- 调用 `get_system_status` 和 `query_process`。
- 追加 2 个 Context Path 节点并打印节点链。

`agent_demo loop`：

- Admin-Agent 设置 30ms 心跳并进入等待。
- 心跳到期后唤醒 Admin-Agent。
- fork Worker-Agent，Worker-Agent 调用 `agent_wait()`。
- Admin-Agent 通过 `send_message` 唤醒 Worker-Agent。
- 打印心跳/消息事件和 Context Path。

`agent_demo fs_query_bench`：

- 创建 `m7_a` 到 `m7_d` 四个文件。
- 设置 `type`、`owner`、`tag`、`priority` 属性。
- 查询 `type=memory AND owner=worker AND tag=social`。
- 输出 `traversal=4 indexed=2` 的访问次数对比。

`agent_demo full`：

- 综合执行心跳唤醒、系统状态查询、文件属性查询、消息唤醒。
- 输出 4 个 Context Path 节点，展示 Admin-Agent 的完整巡检路径。

## 调试记录

第一次构建 `fs-img` 失败，原因是共享代码文件放在 `rcore/user/src/bin/agent_demo_shared.inc`，`easy-fs-fuse` 会扫描 `src/bin/` 下所有文件并尝试寻找同名 ELF。修复为把共享代码移动到 `rcore/user/src/agent_demo_shared.rs`，`src/bin/` 下只保留真实可执行入口。

QEMU 自动化验证使用管道输入 usershell 命令。所有测试程序输出 `passed` 后，usershell 继续等待下一条输入，因此外层 `timeout` 最终终止 QEMU；退出码为 `124`，但 demo 和回归输出已经完整通过。

## 关键文件

- `rcore/user/src/agent_demo_shared.rs`
- `rcore/user/src/bin/agent_demo_basic.rs`
- `rcore/user/src/bin/agent_demo_loop.rs`
- `rcore/user/src/bin/agent_demo_fs_query_bench.rs`
- `rcore/user/src/bin/agent_demo_full.rs`
- `rcore/user/src/bin/user_shell.rs`

## 已知限制

- 当前 rCore 用户态没有 argv，`agent_demo <mode>` 由 shell 映射到四个具体二进制实现。
- demo 中的 Agent 决策策略是确定性的 mock policy，不接入外部 LLM。
- 文件属性表仍沿用 M6 的内存态设计，重启后不保留。
