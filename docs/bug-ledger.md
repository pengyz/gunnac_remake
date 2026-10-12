# Bug 台账（docs/bug-ledger.md）

> 用途：登记**原版（ROM/oracle 行为）的 bug 与 quirks**。按 AGENTS.md D4/§6：
> Stage A **保真保留**原版行为，只登记不修；代码侧以 `// BUG-xxx: 保留原行为` 引用；
> 差分测试遇到已登记 bug = "已知偏差"，基线显式 TOLERATED 并注明 BUG 编号。
>
> 内容维护归 **verifier**（master-plan §4）；本 stub 由 captain 于 2026-10-12 创建
> （轮次日志 round-007，t3 门禁 docs-links 预扫描发现 AGENTS.md §6 引用本文件缺失，
> 裁定方案 A 补齐脚手架）。

## 登记格式（AGENTS.md §6）

```markdown
## BUG-xxx: <一句话现象>
- 现象: <可观察行为描述>
- 复现: <scenario/输入序列/ROM 偏移>
- 影响: <对差分验证/玩法的影响>
- 证据: <trace id | bank:addr | rom:offset | shot:file>
- 处理: Stage A 保真保留（代码注释引用 BUG-xxx）；Stage B 修复计划：<无/说明>
- 登记: <日期 / 登记人>
```

## 台账（当前为空 —— 尚无已证实的原版 bug 登记）

<!-- 新登记追加在下方，按编号递增；证伪不删历史，原地标注日期更正（AGENTS.md §3.6） -->
