---
type: reference
title: 门禁子命令速查 + SKIP/TOLERATED 语义（SKIP ≠ 通过）
date: 2026-10-12
status: confirmed
---

# reference：gates/cli.py 用法与 SKIP/TOLERATED 语义

## 现象

项目早期 `gates/cli.py` 尚未建立，任务完成判据里的门禁命令跑不了。两种做法并存过：
"声称通过"（错）与"如实标 SKIP + 贴人工可核验最小证据"（对，round-005 判定为"合规 SKIP"）。
此外 cli.py 落盘后暴露其子命令**多于 README 表**（round-008 §②：baseline/selftest/show/tolerate/update），
说明"用法速查"需要随时可查、且要留意 README↔cli 同步。

## 速查

| 子命令 | 用途 | 时机 |
|---|---|---|
| `python3 gates/cli.py status` | 只读预检，显式打印 SKIP/TOLERATED | 动手前 |
| `python3 gates/cli.py check` | 总闸（pre-commit 自动跑同一套） | 任何 commit 前 |
| `python3 gates/cli.py all` | 全量 | 里程碑收尾 |
| `rom` / `seed` / `naming` / `trace` / `visual` | 单项门禁 | 对应改动后 |
| `baseline` / `selftest` / `show` / `tolerate` / `update` | 基线与自检（超出 README 表） | 用前先查 README↔cli 同步 |

**输出语义**：

- **SKIP ≠ 通过**：SKIP = 检对象不存在或门禁当前不可跑。必须显式打印，绝不静默通过（AGENTS.md §5）。
  `check` 输出自带声明："CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）"。
- **TOLERATED** = 显式列出的已知偏差，必须注明 BUG 编号（AGENTS.md §6）；视觉白名单是唯一允许的例外清单，
  变更需用户确认并记录。
- **FAIL**：按 AGENTS.md §4 定位层修根因；禁止调阈值/加白名单/删门禁掩盖；门禁改动单独 commit。

**门禁不可跑时的合规姿势**（prompts/README §5）：如实标 SKIP + 给"人工可核验最小证据"
（贴实际执行的命令与输出，如 `grep -abo` / `sha256sum -c`）。**完成 ≠ 声称**（round-005）。

## 为什么

门禁是唯一裁判（AGENTS.md §1）；SKIP 的语义边界一旦被"顺手当通过"，
整条差分防线静默失效——而显式打印 SKIP 至少让缺口在每轮日志里可见、可排期补。

## 何时使用

- 写任务完成判据/跑验收/读门禁输出时；任务卡的 Verify 命令跑不动时按"合规姿势"处理。
- 读 `check` 输出先看 SUMMARY 的 SKIP 数与 TOLERATED 列表，再看绿红。
- 引用本速查前验证 `gates/cli.py` 子命令仍与 README 一致（round-008 §② 的提醒）。

## 证据指针

- `AGENTS.md` §5（SKIP 与 TOLERATED 必须显式打印；门禁表；失败处理）
- `docs/rounds/round-005.md`（合规 SKIP 判例："如实标 SKIP + 人工可核验最小证据——完成≠声称"）
- `docs/rounds/round-008.md` §②（cli.py 子命令超出 README 表 → 验收必查 README↔cli 同步）
- `gates/cli.py`（`gate_knowledge_format` 等的 PASS/FAIL/SKIP 返回与"0 检对象"显式打印语义）
