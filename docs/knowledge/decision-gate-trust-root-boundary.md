---
type: decision
title: 门禁信任根边界：三文件自洽伪造不可内圈检出——不造内圈，记为已知边界（Stage B 再议外部信任根）
date: 2026-10-12
status: confirmed
---

# decision：门禁信任根边界（t15 O1）——接受为已知边界，不再造内圈

> 任务指定路径为 `docs/knowledge/gate-trust-root-boundary.md`；因 knowledge-format 门禁要求
> 文件名以 `<type>-` 前缀开头（判定权在门禁），落盘为 `decision-gate-trust-root-boundary.md`，语义名保留于标题。

## 现象

t15（review-round-3，verdict=pass）复审观察 O1：**三文件完全自洽伪造可骗过全部内圈校验**——
把 `baseline.json`、`baseline_history.jsonl`（含伪造整条审计链、fake reason）、`SHA256SUMS`
**同时**伪造成相互一致，则 baseline-integrity / rom-integrity / ratchet 等内圈检查全部绿。
t15 明确定性：**文件信任根固有边界，非实现缺陷**。

同轮对照实测（边界之外的攻击均已拦）：t11 攻击链"删历史 + 手改基线 + 重写 SHA256SUMS"等
**非自洽/单侧改写**被三重检出（exit 0→1）；历史删/空/部分留/单字节篡改、损坏 JSON 裸 traceback、
降基线无痕提交等全部被拦（round-012 §① 攻防表）。

## 为什么

- 内圈校验的判定依据**就是这三份文件本身**：它能证明的是**内部一致性**——历史存在且非空、
  逐 metric 有历史下界、sha 链与状态锚点一致、敏感路径提交含 Evidence/Confidence。
- 一致性校验**原理上无法区分**"真实演进"与"伪造的自洽演进"：伪造者同时重写全部状态及其哈希锚点时，
  验证者没有任何**第三方参照**可比对——这就是**文件信任根**的固有边界。
- 所以再造内圈（更多一致性检查）只抬高伪造成本一点点，不改变边界性质；
  verifier 建议**入 knowledge 记录而非再造内圈**，captain 照准（t15 → round-012 §①）。

## 何时使用（含缓解：现状兜底与 Stage B 再议）

- **现状双兜底**：①降基线/改基线提交必含 `Evidence`/`Confidence`（commit-msg 强制，
  `data/status/` 已入敏感路径，t14 实测拒/放行）；②**git 评审**（commit 历史人审 + 轮次日志留痕）。
- 遇到"门禁全绿但基线可疑"的质疑：按本条**先问信任根**（三文件是否可能被同时伪造），
  **不要把内圈绿当作真实性证明**；取证用外部可核对物——攻击复现在 /tmp 副本 + 临时 git 仓库
  （t15 方法：取证后清理，共享仓库不污染）。
- **进一步收紧**（外部信任根：CI 侧锚点/签名/远端只追加日志）→ **Stage B 再议**，Stage A 不投入。
- 关联待办（不在本条范围）：O2 文案 nit——baseline-integrity OK 行"（零留痕改写可检出）"
  措辞对本场景过 claim，应限定为"非自洽/单侧改写可检出"，归下次门禁 commit 顺手修。

## 证据指针

- **t15 output**（任务 t15 review-round-3，attempt 0938861c，verdict=pass，观察 O1 原文：
  "三文件完全自洽伪造（伪造整条审计链含 fake reason）会绿——文件信任根固有边界，非实现缺陷，
  兜底=降基线提交必含 Evidence/Confidence（已强制）+ git 评审；若未来要再收紧需外部信任根
  （CI 侧锚/签名），建议记入知识库而非再造内圈"）
- `docs/rounds/round-012.md` §①（O1 处置：派 t16 落知识条目，verifier 建议照准；O2 归门禁 commit）
  、§③（t16 看板行）
- `data/status/baselines/`：信任根三文件 `baseline.json` / `baseline_history.jsonl` / `SHA256SUMS`
- `gates/cli.py` `baseline-integrity`（内圈校验面：历史非空/逐 metric 下界/sha 链/锚点一致；
  其 OK 行文案即 O2 对象）
- commit `c314e10`（`data/status/` 入 commit-msg 敏感路径，AGENTS.md §5/§9 更正注——兜底一环）
- commit `5fc2533`（基线机制入库 + master-plan §5.5 基线政策修订）
