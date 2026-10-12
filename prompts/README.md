# prompts/ — 角色提示词与提示词纪律

新任务开工前**必须先选定角色 prompt**，把它作为该任务的完整工作契约（AGENTS.md §7）。
角色 prompt 是任务级提示词的模板；用户的具体任务以"任务卡"形式追加在角色 prompt 之后。
全自动回路（[docs/ai-decompile-pipeline.md](../docs/ai-decompile-pipeline.md) §4）中，
任务卡由完备性矩阵派生，同样遵循本模板。

## 角色一览

| 角色 | 文件 | harness 技能（稳定调用入口） | 适用阶段 | 产出归宿 |
|---|---|---|---|---|
| 反编译分析员 | [decomp-analyst.md](decomp-analyst.md) | `remake-decomp-analyst` | S1–S3 投影/命名/数据表考证 | `data/naming/` 台账 + `reference/` + `docs/findings/` |
| 规格作者 | [spec-writer.md](spec-writer.md) | `remake-spec-writer` | S4 理解逻辑 | `docs/spec/` |
| 改写工程师 | [porter.md](porter.md) | `remake-porter` | S5 Rust 移植 | `crates/` |
| 差分验证员 | [verifier.md](verifier.md) | `remake-verifier` | S6 验证/门禁 | `gates/` + `data/status/` + bug 台账 |

> **技能同步规则**：同名 skill（`~/.agents/skills/remake-*/SKILL.md`）是 harness 的稳定调用入口，
> 本目录文件是仓库内完整契约。修改任一侧必须同步另一侧；冲突时以本目录 + AGENTS.md 为准。

## 提示词纪律（所有角色通用）

1. **任务卡最小要素**：目标、范围（可改路径）、证据输入（已有 findings/台账条目）、完成判据（跑哪个门禁命令）。
2. **产出必须包含**：①证据引用（A/B 级，见 AGENTS.md §3）②完成判据的实际执行结果 ③遗留假设清单（C 级）。
3. **产出的归宿是仓库**：台账 / spec / 代码 / 知识库 / bug 台账。聊天记录里的结论**不许**作为后续任务依据。
4. **禁止事项继承**：角色 prompt 中的"禁止"叠加生效于 AGENTS.md 的禁令，不互相覆盖。
5. **完成≠声称**：角色任务结束前必须实际运行完成判据的门禁命令并贴输出（AGENTS.md §5）；
   门禁未启用（SKIP）时，完成判据退化为"人工可验证的最小证据"并在产出中注明。
6. **角色切换**：一个任务卡只用一个角色。需要切换角色（如移植中发现理解错误）→ 停下，
   按 AGENTS.md §4 三问定位层，另开对应角色的任务卡，禁止混着做。
7. **投影不背书**：IDA/Ghidra 的反汇编与伪代码是线索不是事实；引用时注明工具与地址，
   与 trace/ROM 字节冲突时以 ROM 为准（AGENTS.md D3/D5）。

## 任务卡模板

```markdown
### 任务卡 <id>
- 角色: <decomp-analyst|spec-writer|porter|verifier>
- 目标: <一句话>
- 范围: <可修改路径；未列出 = 不许改>
- 证据输入: <findings/台账/汇编地址 bank:addr/trace id/代码行指针>
- 完成判据: `python3 gates/cli.py <gate>` 全绿 + 产出物清单
- 禁止: <本任务特有禁令>
```
