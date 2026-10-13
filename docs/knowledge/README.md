# docs/knowledge/ — 跨会话知识库

> 用途（AGENTS.md §8）：跨会话共享已确认的知识——平台坑、门禁失效、被纠正的方法、确认有效的取舍。
> **写入的是"已确认的教训"，不是"看起来有理的猜想"**；引用任何条目前必须验证其证据指针仍然成立。

## 1. 六类（type，文件名前缀 = type）

| type | 收什么 | 反例（不收） |
|---|---|---|
| `architecture` | 结构/层次/数据流的长期事实 | 一次性调试记录 |
| `gotcha` | 坑与反直觉陷阱（做 X 会静默得到坏结果） | 尚未踩实的担忧 |
| `pattern` | 反复验证有效的做法/协作机制 | 只成功一次的尝试 |
| `debug` | 排查诊断经验（症状→定位→修复） | 与诊断无关的用法说明 |
| `decision` | 决策及理由（含被否决的选项） | 未定论的讨论 |
| `reference` | 事实速查（命令/格式/语义/约定） | 教程长文 |

## 2. 写入规则（准入硬约束）

1. **文件名**：`<type>-<kebab-case-标题>.md`（门禁校验：文件名必须以 `type` 值 + `-` 前缀开头）。
2. **frontmatter**：文件必须以 `---` 开头的 frontmatter 块开始，且含 `type:` 行（值 ∈ 六类）。推荐字段：

   ```markdown
   ---
   type: gotcha
   title: 一句话标题
   date: YYYY-MM-DD
   status: confirmed          # confirmed | superseded
   ---
   ```

3. **正文结构**（缺一不可）：`现象` / `为什么` / `何时使用` 三节 + `证据指针`。
4. **证据指针**：必须指向可复现物——轮次日志（`docs/rounds/round-NNN.md` §小节）、commit hash、
   命令输出、门禁输出。无指针不入条目（同 AGENTS.md §3 证据纪律）。
5. **只收已确认教训**：假设/猜想进 `docs/findings/` 假设区；被推翻的条目**原地加日期更正**
   （`> 【YYYY-MM-DD 更正】…`），`status: superseded`，**不删历史**。
6. **引用前必须验证**：命令/路径/文件是否仍存在（AGENTS.md §8）；引用失效 = 引用者义务。
7. 知识条目**不写实现细节**（代码怎么写），只写可迁移的判断与事实。

## 3. 门禁

- `gates/cli.py check` 的 **knowledge-format** 项校验：frontmatter 存在、`type` 合法、文件名前缀匹配。
  建立本目录后该项从"0 检对象"转为实际校验。
- `knowledge_entries` ratchet 计数只升不降（新增条目合法；删除条目需 commit 说明）。

## 4. 条目清单

| 条目 | type | 一句话 |
|---|---|---|
| [gotcha-reusable-tools-belong-in-tools.md](gotcha-reusable-tools-belong-in-tools.md) | gotcha | 可复用工具入 `tools/`（入库），`analysis/` 只放可再生产出——同类路径坑已踩两次 |
| [pattern-cross-scope-conflict-escalation.md](pattern-cross-scope-conflict-escalation.md) | pattern | 触碰 inScope 之外的文件：不越界不变通，带选项上报裁定 |
| [gotcha-rate-limit-is-infrastructure-not-diff-routing.md](gotcha-rate-limit-is-infrastructure-not-diff-routing.md) | gotcha | 429/限流是基础设施故障，不进四分类差异路由 |
| [reference-gate-usage-and-skip-semantics.md](reference-gate-usage-and-skip-semantics.md) | reference | 门禁子命令速查 + SKIP/TOLERATED 语义（SKIP ≠ 通过） |
| [pattern-evidence-spotcheck-independent-reproduction.md](pattern-evidence-spotcheck-independent-reproduction.md) | pattern | 验收 = 独立复现证据指针，不看报告写得对不对 |
| [decision-baseline-raise-policy.md](decision-baseline-raise-policy.md) | decision | 基线分级：进度类上调 captain 验收后代执行；下调/放宽/白名单仍须用户确认 |
| [decision-gate-trust-root-boundary.md](decision-gate-trust-root-boundary.md) | decision | 信任根边界：三文件自洽伪造不可内圈检出——不造内圈，Evidence/Confidence+git 评审兜底，Stage B 再议外部信任根 |
| [gotcha-parallel-read-image-order-unreliable.md](gotcha-parallel-read-image-order-unreliable.md) | gotcha | 并行多图 read_image 结果序不可信——多图单张调用或字节级锚定，数值结论禁只凭读图 |
| [gotcha-concurrent-mesen-output-corruption.md](gotcha-concurrent-mesen-output-corruption.md) | gotcha | 并发 Mesen 共写输出=幻影输入/假 trace——串行锁+归档+自管 Xvfb，外来位先查并发污染（并行不可信家族） |
| [pattern-visual-binding-needs-pixel-proof.md](pattern-visual-binding-needs-pixel-proof.md) | pattern | 视觉↔RAM 绑定必须像素级证明+分离事件检验——相关性≠绑定（N-0032→N-0037 教训） |
| [gotcha-git-add-commit-atomicity.md](gotcha-git-add-commit-atomicity.md) | gotcha | git 索引并发卷入——精确 pathspec+add/commit 原子窗口，误卷不删历史（并行不可信家族第三员） |
| [gotcha-observation-granularity-fabricates-invariants.md](gotcha-observation-granularity-fabricates-invariants.md) | gotcha | 观测粒度伪造不变量——"同帧同步"声明必须绑定粒度（R-bomb-2 证伪链 f2174/f2175） |
| [gotcha-stack-page-game-var-watch-coverage.md](gotcha-stack-page-game-var-watch-coverage.md) | gotcha | 栈页 $0100-$01FF 是游戏变量密集区——watch/写集按变量地址逐页核对，"0 断言"先查采样面 |
| [gotcha-false-green-idle-assertions.md](gotcha-false-green-idle-assertions.md) | gotcha | 断言存在≠断言生效——解析层级错致断言空转（caed518 ss-1/2），新增断言必做负向自证 |
| [gotcha-register-address-is-not-nametable-target.md](gotcha-register-address-is-not-nametable-target.md) | gotcha | 注册器地址≠nametable 目标——$2006 锁存+$2007 自增协议暗雷（十一击共同根因），消费 PPU 写日志先还原落点 |
| [gotcha-cpu-file-offset-conversion.md](gotcha-cpu-file-offset-conversion.md) | gotcha | CPU↔file 换算显式声明——漏 iNES 头 $10 的 off-by-16 陷阱（t149 复审活例），双偏移对账三件套 |
| [pattern-repair-selfcheck-grep.md](pattern-repair-selfcheck-grep.md) | pattern | 修复自查纪律——grep 目标行+新引入事实复核（三轮"声称已修"教训），修复≠声称修复 |
