# M6 Design

## 目标

M6 实现简化文件属性查询扩展，对应 `Docs/PLAN.md` 中的 “File Attribute Query”：

- 内核维护内存态文件属性表。
- 支持为文件绑定 `type`、`owner`、`tag`、`priority` 四类属性。
- 支持属性删除。
- Tool Call 新增 `query_file`，按多条件 AND 查询文件。
- 查询结果写入 Agent Context，并携带遍历查询与属性索引查询的访问次数对比。

## 属性表

M6 不修改 easy-fs 的磁盘 inode 格式，而是在内核中新增固定容量内存表：

```text
FileAttrEntry {
  path[32], path_len,
  file_type,
  owner,
  tag,
  priority
}
```

表容量为 16 项，适合当前教学 demo。属性在重启后丢失，后续如需持久化可扩展为 inode xattr 或独立属性文件。

## 系统调用

新增 syscall：

- `511`: `sys_file_attr_set(FileAttrSetRequest*)`
- `512`: `sys_file_attr_delete(path)`

`sys_file_attr_set` 会先检查目标文件是否存在，再新增或覆盖属性项。路径最长 32 字节，返回码沿用 Agent 工具错误码：

- `0`: OK
- `-5`: 参数错误，如路径为空或过长
- `-6`: 属性表容量不足
- `-7`: 文件不存在或待删除属性不存在

## Tool Call

新增工具：

- `TOOL_QUERY_FILE = 4`

参数沿用 M3 的固定 `u64` 参数协议：

- `TOOL_PARAM_FILE_TYPE = 20`
- `TOOL_PARAM_FILE_OWNER = 21`
- `TOOL_PARAM_FILE_TAG = 22`
- `TOOL_PARAM_FILE_PRIORITY = 23`

多个参数之间为 AND 关系。结果结构：

```text
ToolFileQueryResult {
  total_matches,
  returned,
  traversal_visits,
  indexed_visits,
  items[8] = ToolFileSummary
}
```

`traversal_visits` 表示全量遍历属性表会访问的属性项数；`indexed_visits` 表示用第一个查询条件作为简化索引键后的候选项数。当前实现用同一张内存表模拟索引访问次数，不额外维护复杂哈希表，优先保证演示稳定和评估指标清晰。

## 兼容性

- `tool_list` 返回工具数量从 3 增加到 4。
- M3 旧测试只要求前三个工具仍存在，不再固定总数为 3。
- 文件查询仍要求调用者是 Agent，因为查询结果会写入 Agent Context。
