# oracle/scenarios/ —— 场景脚本 DSL

机器裁定的输入轨道：每个 `*.txt` 是一个**确定性场景**（输入序列 + 帧区间 + 采集点），
由 `tools/scenario_run.py`（DSL 唯一解析器）驱动 Mesen2 2.1.1 采集 trace/截图。
方法论地位见 AGENTS.md §2 `oracle/` 行与 D3：这里是**参考轨道**（原 ROM 行为的证据来源），
比对逻辑不得为"通过测试"放宽阈值。

## 运行

```bash
python3 tools/scenario_run.py list            # 列场景
python3 tools/scenario_run.py run <场景名>     # 跑一个（产物见下）
python3 tools/scenario_run.py run-all         # 全部顺序跑
```

内部等效命令（已验证的装载方式）：
`xvfb-run -a tools/mesen2/Mesen rom/gunnac_cn.nes analysis/_gen/<场景名>.lua /DoNotSaveSettings`

## DSL 指令（一行一条，`#` 后为注释）

| 指令 | 语义 |
|---|---|
| `name <名称>` | 场景名（缺省=文件名 stem；产物文件名前缀） |
| `frames <N>` | 总帧数（必填；帧号 1..N） |
| `press_len <N>` | `press` 按住帧数（缺省 2） |
| `press <帧> <按钮...>` | 帧 f..f+press_len-1 按住，之后释放（点按） |
| `hold <起>-<止> <按钮...>` | 帧 起..止（含）按住，帧 止+1 释放 |
| `capture <帧> <tag>` | 该帧末截图 + 全量 2KB RAM dump + cpu 寄存器 |
| `capture_every <N> <tag>` | 每 N 帧做一次 capture |
| `watch <hex地址> [标签]` | 每帧导出该字节（标签缺省=地址；语义命名需台账证据） |
| `watch_range <hex起>-<hex止> [标签]` | 每帧导出该区间 hex |
| `ramframe on\|off` | 每帧导出全量 2KB RAM hex（缺省 off；短窗口用） |

按钮集：`a b start select up down left right`（Mesen2 `emu.setInput` 键名）。

## 帧计数与时序约定（差分复现靠它锁死）

**Q4 输入注入 +1 帧延迟约定（t28 实证，12/10 边界差异全为此类）**：
场景 DSL 的 `press`/`hold` 在 **schedule 帧 N** 注入按键，而游戏侧 `controller_held`（$18）
**反映于帧 N+1**（trace 中 endFrame N+1 的 watch 才看到该位）。等价说法（L3 对齐约定）：
`state(N) = tick(state(N-1), held(N))` —— 当帧 trace 的 held 驱动进入下一状态的转移。
两个说法同源：schedule(N) == trace.held(N+1)。改输入脚本或改差分对齐时勿混用两种计数。

- `frame` 在 `inputPolled` 回调递增：第 N 次 poll = 帧 N，该帧注入 `schedule[N]` 的按键；
- trace 行 / 截图 / RAM dump 在 `endFrame`（帧 N 渲染完成后）产生；
- `frame == frames` 时写 `<场景名>.done` 标记并停机；驱动等标记后 SIGTERM 收尾。

## 产物

| 路径 | 内容 |
|---|---|
| `analysis/traces/<名>.frames.jsonl` | 逐帧状态向量 `{"f":N,"w":{标签:字节/区间hex},"ram":"…"?}` |
| `analysis/traces/<名>.captures.jsonl` | 采集点索引：帧号/tag/截图路径/RAM hex 路径/cpu 寄存器 |
| `analysis/traces/<名>_fNNNNN_<tag>.ram.hex` | 采集点全量 2KB RAM（4096 hex 字符） |
| `shots/<名>_fNNNNN_<tag>.png` | 截图（emu.takeScreenshot） |
| `analysis/traces/<名>.run.json` | 运行清单：场景/生成 lua 的 sha256、计数、耗时（可复现证据） |

`analysis/`、`shots/` 均为可再生工作区（.gitignore），**场景 *.txt 是入库的真相源**；
引用产物时给复现命令 + run.json 里的 sha256。

## 证据纪律

- `watch` 标签缺省是地址本身（如 `0320`）。**语义命名（player_x / money 等）必须先有台账证据**
  （AGENTS.md §3：A/B 级才准改名；不确定用 `_todo_` 或保留地址名）；
- 场景文件里的注释性猜测必须标注"假设"，不得写成结论；
- 场景输入脚本本身是行为假设的载体：改输入 = 改证据轨道，需单独 commit 说明。

## 现有场景

| 场景 | 内容 | 状态 |
|---|---|---|
| `title-menu.txt` | 开机 → 标题 → 菜单流程（START/A 推进，f360 标题、f540/f720/f900/f960 菜单族转迁） | 已跑通，逐 30 帧留证 |
| `shop.txt` | 进入商店（f1080）→ A/B 切页采样（f1140 信息页"导弹20个上限"） | 已跑通，交互语义见 `docs/spec/shop-economy.md` R-shop-* |
| `gameplay-stage1.txt` | 商店退出（两连 B）→ 实机：自动移动 + 每 12 帧自动射击 + 炸弹采样 | **已补采实机画面**（矩阵 §2.6 入口） |

## 已证事实（S2 首批，exp-shop-b/start/nav 三连探索 + 上述场景复核）

| 事实 | 级 | 证据 |
|---|---|---|
| f180 黑屏 → f360 标题 → f540/f720/f900/f960 菜单族 → f1080 商店(efcfe736) | B | 三连 exp 场景 sha 转迁表一致（`analysis/traces/exp-shop-*.run.json` 记 DSL sha） |
| 商店稳定屏为静态：无输入 2000 帧画面不变 | B | `exp-shop-start` 全程 sha 不变（Start 无效） |
| 商店内 B 切信息页（f1140："你有———￥0 / 导弹20个 上限。" + "A键,B键转换"） | A | `shots/exp-shop-b_f01140_probe.png`（sha fec87b88） |
| 退出商店进实机：商店内两连 B（f1100、f1160）→ f1200 近黑过渡 → f1260 起实机 | B | `shots/exp-shop-b_f01260..f01980` 星域+自机+敌机+HUD；gameplay-stage1 场景复现 |
| Mesen2 Lua：`emu.read(addr, memType=8)` = CPU 总线视图；截图 `emu.takeScreenshot()`；进程不随 `emu.stop` 退出（驱动看 DONE 后 SIGTERM） | A | t8/t12 探针 `/tmp/mesen_probe*`（重跑脚本见任务记录） |

> 运行纪律：**场景必须串行运行**（并行多个 Mesen 实例会互扰——t12 实测；驱动已改为自管
> Xvfb + DONE 标记 + 进程组清理）。t24 事故（并行重跑踩踏他人在用的 trace）后新增两道防踩踏：
> **串行锁** `analysis/.scenario.lock`（同时只允许一个场景运行）+ **覆盖前时间戳归档**
> （旧产物移入 `analysis/traces/.archive/<名>-<时间戳>/`，run 级隔离、历史可查）。
> L3 golden（`oracle/traces/<场景>.player.jsonl`）只从确定性 run 抽取（重跑 sha256 一致才可信），
> 死亡/重生等未证语义窗口按 spec §4 显式剔除并在抽取脚本登记帧域。
