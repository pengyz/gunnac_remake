# 对抗复核 #98 — 域裁定三线 + S4b-2x 批1/2 + 账面（小复核·积压预防）

> 日期：2026-10-16｜角色：adversarial-reviewer（证伪立场；实施方=多棒≠审核方）
> 被审：域裁定三线（bot 域独立 golden 80afee4/5f67631、注入域 6 行回填 ab40ba1、OBS-1 封存维持）
> + S4b-2x 批1/2（0530be1/d02d055 bank3 六骨架）+ 五鸟账 rev 40。
> 三证伪靶：①6 行回填值域来源核（stg01-chain 16/17 真锚）②bot 域 golden 分域注 ③骨架形态抽 2 ROM 重导。
> 拆段落盘（防断照旧）。

## 段 A：①6 行回填值域来源核——**6/6 逐值=锚台账真值** ✅

### A.1 回填链核（ab40ba1 `golden_backfill.py --cross-scene`）

| 行（stg01-chain.l5.jsonl 内 scene=stg02-08-chain） | 回填 f | 锚台账（s5-all-clear.anchors.jsonl）真锚 | 裁定 |
|---|---|---|---|
| stg02-start StageStart | 3450 | s5-stg01 stg02-start f=**3450** | ✅ |
| stg03-start | 5700 | s5-chain stg03-start f=**5700** | ✅ |
| stg04-start | 7500 | 7500 | ✅ |
| stg05-start | 9600 | 9600 | ✅ |
| stg06-start | 11700 | 11700 | ✅ |
| stg07-start | 14600 | 14600 | ✅ |

**6/6 逐值全等** ✓✓；第 7 占位（stg08-start）锚台账自身 f=0（无真锚）⇒ **不回填 fail-closed**
（不发明 ✓ 与工具"占位无台账锚"输出一致）。
**账面自证**：工具直跑输出 `stg01-chain 真锚 16/17` ✓✓ 与声称"16/17 真锚"精确一致
（s5-all-clear 面 `真锚 39/44` 同口径）。
跨场景 identity 断言（checkpoint+mode 精确、scene 不同源共享链前缀帧域）与实现
（`ai[(checkpoint,mode)]` 索引+f>0 过滤）逐行吻合 ✓——分域不混账（注入域 1260..14600）。
