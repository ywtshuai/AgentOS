# M8 Design

## 目标

M8 补齐 `Docs/PLAN.md` 的最终文档交付项，把 M1-M7 的实现整理成评审可直接阅读的入口：

- 更新根目录 `README.md`，包含编译方式、QEMU 运行方式、demo 命令和 syscall/tool 列表。
- 新增 `Docs/design.md`，描述 Agent-OS 总体架构、Agent 进程模型、地址空间布局、Tool Call 协议、Context Path、Agent Loop 和文件查询扩展。
- 新增 `Docs/evaluation.md`，汇总功能测试、综合 demo、文件查询性能对比和已知限制。
- 保留每个里程碑的 `DESIGN.md`、`IMPLEMENTATION_LOG.md`、`TEST_REPORT.md` 追溯链。

## 文档结构

```text
README.md
Docs/design.md
Docs/evaluation.md
Docs/M8/DESIGN.md
Docs/M8/IMPLEMENTATION_LOG.md
Docs/M8/TEST_REPORT.md
```

## 内容边界

M8 是交付文档里程碑，不改变内核、用户态 ABI 或 demo 行为。测试以文档格式检查、git diff 检查和现有构建/测试记录复核为主。

## 与计划对应

M8 对应 `Docs/PLAN.md` 中的 Documentation Deliverables：

- `README.md`
- `docs/design.md`
- `docs/evaluation.md`

仓库已有 `Docs/` 目录，因此最终文件使用 `Docs/design.md` 和 `Docs/evaluation.md`，保持路径风格一致。
