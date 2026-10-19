# 对抗复核：C3 敌坐标实现（task-164）— task-167

> 【2026-10-19】复核执行说明：原指派 shell-verifier 的复核 turn 因队友/subagent 基建故障连续崩溃
> （5 个 turn 无收尾），由 **Lead 亲自独立重导全部验证**（实施方=shell-porter ≠ 复核方=Lead；
> 独立 mimo 子 agent 复核亦尝试失败）。以下全部数字为 Lead 自跑复现，命令与输出摘要如实附列。

## VERDICT: **pass**（附 2 条非阻塞注记）

## 独立验证（Lead 自跑，2026-10-19）

| 项 | 命令 | 结果 |
|---|---|---|
| 全量测试 | `cargo test -p game-shell -p shell-win` | **全绿**：game-shell lib 61 / contract 2 / enemy_coords_replay 2 / mock 3 / stg01_combat 1 / stg01_replay 3；shell-win lib 14 / smoke 23；**0 failed** |
| §4.2 对拍复算 | `cargo test -p game-shell --test enemy_coords_replay -- --nocapture` | `C3 共同采样点 N=199；x≤1px=199/199 (100.0%)；y 0容差=199/199 (100.0%)；$2B 配对段=12（含 reuse 段 7-12）`，`test result: ok. 2 passed` |
| selftest 确定性 | `cargo run -p shell-win -- --selftest-frames 300` ×2 | 签名行两次一致 ✓（host `0x68547665bf1ab396` / canvas `0x963ac9a776a57625` / display `0xe99af5849b632fbd` / view `0x15dbc2c7b0ba0a5d`） |
| 四签名不变原因 | 读 host.rs 签名函数 + selftest 路径 | selftest 无装载 ⇒ 无 spawn；`Host::signature()` 仅哈希槽型值不含坐标模型 ⇒ 四签名不变**成立**（复核通过，非回退掩盖） |
| 不可对拍清单 | 测试 stdout | `oracle 额外型 $12/$1A/$1C/$2A（生成源未接，不判 FAIL）`、`GUI $2B 段#12-14（无敌人释放⇒网格饱和，fail-closed）`——如实登记、未判 FAIL、未发明释放语义 ✓ |

## findings

- **证据合规（pass）**：$2B init（y=8/vy pair `$588=$90,$5A4=$00`/x=param `$C599` 链或 `$18/$E8` 按 `$2D` 奇偶 B 级/目标 `$5F8`）与 tick（`$CD63 ±#$20` + `$CD08/$CCD3` 有符号积分）与 `enemy-coords-c3-audit.md`（task-157/161/162/163 pass）一致；reuse 复位（槽型 0→非0 + 延迟一帧、leader 落地事件、`$0470`=drip param 保留）符合 F1 口径；$4E 运动 fail-closed（`TODO(c3-vm)`）、无发明图样。
- **实现错误（pass）**：`motion_update` 无符号积分对负速度误判 Offscreen 的问题由模块内有符号积分绕开并注明——处理正确。
- **规格缺口（P3 注记）**：spawn 分段对齐口径（"精确 spawn 帧不可比"显式登记）与 spec §4.2 原样的差异已在 plan §9.2/§9.3 登记；分母 assert_eq! 冻结（N=199=98+96+5）。非阻塞。
- **平台差异（pass）**：无跨平台假设混入。

## 修正建议（非阻塞）

1. 成员复用段 13-24 的 Rust 对应需 `$D616` 释放语义解码（另立取证卡后接入）；
2. `$2B` param=0 的 `$2D` 奇偶为 B 级模型，后续可用 GUI trace 逐值收敛为 A 级。

## 结论

实现与审计证据一致、对拍数字可复算、fail-closed 边界未越、测试全绿、签名变化零（原因如实）。
**verdict = pass**，可入库（随 task-164/167 提交）。
