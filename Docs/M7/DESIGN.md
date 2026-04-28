# M7 Design

## 目标

M7 实现 “Agent 系统管理员” 综合演示，对应 `Docs/PLAN.md` 中的 Integrated Demo：

- 整合 Agent 创建、Agent Context、Tool Call、Context Path、心跳/消息唤醒、文件属性查询。
- 提供可在 usershell 中重复运行的 demo 命令。
- 输出每轮 Agent Loop 的关键事件和 Context Path 节点。
- 输出文件属性查询与遍历查询的访问次数对比。

## Demo 命令

rCore 当前 `sys_exec` 只接收程序路径，不支持 argv。为了保留计划中的演示命令形式，M7 在 `user_shell` 中加入轻量命令映射：

```text
agent_demo basic          -> agent_demo_basic
agent_demo loop           -> agent_demo_loop
agent_demo fs_query_bench -> agent_demo_fs_query_bench
agent_demo full           -> agent_demo_full
```

实际 demo 程序仍是普通用户态 ELF，便于文件系统镜像打包和独立回归。

## 场景拆分

- `basic`：创建 Admin-Agent，调用 `get_system_status` 和 `query_process`，把两轮结果摘要写入 Context Path。
- `loop`：Admin-Agent 设置心跳并等待唤醒，随后创建 Worker-Agent，通过 `send_message` 唤醒 Worker-Agent，并记录 Context Path。
- `fs_query_bench`：创建 4 个测试文件并设置属性，调用 `query_file(type=memory AND owner=worker AND tag=social)`，输出 `traversal_visits` 与 `indexed_visits`。
- `full`：串联心跳、系统状态查询、文件属性查询、消息唤醒和 Context Path 展示，作为最终现场演示入口。

## 用户态结构

新增共享 demo 代码：

- `rcore/user/src/agent_demo_shared.rs`

新增四个薄入口：

- `agent_demo_basic.rs`
- `agent_demo_loop.rs`
- `agent_demo_fs_query_bench.rs`
- `agent_demo_full.rs`

共享代码通过 `include!("../agent_demo_shared.rs")` 被四个入口复用，避免复制四套 Agent Tool Call 和 Context Path 辅助函数。

## 与比赛要求的对应关系

`full` demo 至少整合以下模块：

- 任务一：Agent 创建与 Agent Context。
- 任务二：结构化 Tool Call。
- 任务三：Context Path。
- 任务四：文件属性查询与性能对比。
- 任务五：心跳等待与消息唤醒。

demo 输出包含 Agent Loop 事件、文件查询访问次数、Context Path 节点链，便于现场说明系统机制。
