# M8 Test Report

## 测试范围

M8 是文档交付里程碑，未修改内核、用户态库或 demo 程序。测试目标：

- 文档文件存在且路径符合仓库约定。
- Markdown 关键内容可搜索。
- 工作区 diff 无尾随空白等格式问题。
- 复核 M7 记录中的完整构建和 QEMU 回归结果。

## 已执行命令

```bash
rg --files Docs README.md
rg -n "sys_agent_create|agent_demo full|query_file|Agent Loop" README.md Docs/design.md Docs/evaluation.md Docs/M8
git diff --check
```

## 结果

- `README.md` 已包含构建、运行、demo 命令、syscall 列表和工具列表。
- `Docs/design.md` 已包含总体架构、Agent 进程模型、地址空间布局、Tool Call 协议、Context Path、Agent Loop 和文件查询扩展。
- `Docs/evaluation.md` 已包含 M1-M6 回归、M7 demo、文件查询性能对比和稳定性说明。
- `Docs/M8/*` 已记录本里程碑设计、实现和测试。
- `git diff --check`：通过。

## 回归依据

M8 未改动代码，功能回归沿用 M7 完整验证结果：

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

文件查询性能对比沿用 M7 记录：

```text
query_file matches=1 traversal=4 indexed=2 first=m7_b
full file_query matches=1 traversal=4 indexed=2
```
