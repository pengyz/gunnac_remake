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

## 候选观察区（不立号——未满足自然链复现条件的可疑现象，仅供后续排查）

### OBS-1 · 末关商店出店转场截断（2026-10-16）

- 现象：STG-07→08 商店光标 3（出店位）按 A → \$32 40→60 + fade 起步（\$04:=\$4D 首档）后 step2 序列分叉（非 77→53→24 标准）→ ~10f 转场突发 → **结构冻结**（\$27 不释/ptr 不切）。
- 复现：`oracle/scenarios/stg08-timeline.txt` run #1/#3（A@3 短按/长按三连确定性）；4 位菜单矩阵：0/1/2=空按、3=唯一出店动作位。
- 不立号原因：链含 7 次 $26 bit4+$0400 注入（trigger=injected），**不可声称原版行为**；需自然链（全通关 bot）复现方可立 BUG-xxx（AGENTS §6）。
- 关联线索：\$32 bit5 置位（疑末关处理中旗，成功链恒 \$40）；fade step2 分叉（疑目标调色板差异）。



<!-- 新登记追加在下方，按编号递增；证伪不删历史，原地标注日期更正（AGENTS.md §3.6） -->

## BUG-021 · C9 bit5 死分支 quirk（波次流头部）

- 现象：波次流 C9 判定链中 `AND #$20 / BEQ +0`（零效果分支）——bit5 被测试但分支目标=下一指令，读写两侧均无效果
- 复现：`python3 tools/wave_stream_decode.py wave 1..8` 任意流头部 C9 位义段；bank00:89xx C9 判定链逐指令（wave-ca-flags.md §6）
- 影响：行为零差异（死代码）；移植时**保留该判定形态**（D4 保真，不许优化删除）
- trace id：无动态面（静态死分支）
- 立账：2026-10-15（decomp-analyst S3-3 发现，captain 按 D4 立账）；Stage A 不修，Stage B 清理候选
