# M8 Implementation Log

## 修改内容

- 更新根目录 `README.md`：
  - 增加项目总览。
  - 增加构建和运行命令。
  - 增加 usershell demo 命令。
  - 增加 Agent-OS syscall 表和 kernel tool 表。
  - 指向总体设计和评估文档。
- 新增 `Docs/design.md`：
  - 汇总 M1-M7 架构。
  - 增加 Mermaid 架构图、地址空间图和 Agent Loop 状态机。
  - 描述 Agent 进程模型、Tool Call 协议、Context Path、Agent Loop、文件属性查询和综合 demo。
- 新增 `Docs/evaluation.md`：
  - 汇总构建检查。
  - 汇总 M1-M6 回归和 M7 综合 demo 结果。
  - 记录文件查询访问次数对比。
  - 说明 Agent Context 直接读取的评估意义和已知限制。
- 新增 M8 里程碑文档目录：
  - `Docs/M8/DESIGN.md`
  - `Docs/M8/IMPLEMENTATION_LOG.md`
  - `Docs/M8/TEST_REPORT.md`

## 关键文件

- `README.md`
- `Docs/design.md`
- `Docs/evaluation.md`
- `Docs/M8/DESIGN.md`
- `Docs/M8/IMPLEMENTATION_LOG.md`
- `Docs/M8/TEST_REPORT.md`

## 行为说明

M8 不修改代码行为。现有演示入口仍为：

```text
agent_demo basic
agent_demo loop
agent_demo fs_query_bench
agent_demo full
```

根 README 现在可作为评审或演示人员的第一入口，`Docs/design.md` 和 `Docs/evaluation.md` 则对应计划中的最终设计与评估交付。
