# 孤儿 run 分岔复核报告（t28）— t22 f1830+ 结论重验 + W8 收口

> 背景（t24 上报）：t12 时代 `xvfb-run` 早退孤儿 run 的 trace 与确定性 run 在 **f1830 后分岔**；
> 防踩踏三件套（commit **b80135d**：串行锁 `analysis/.scenario.lock` + X display 锁 + 旧产物归档
> `analysis/traces/.archive/`）落地前的产物均属可疑区。本报告用**现行确定性驱动**（`tools/scenario_run.py`
> 串行锁版）重跑 gameplay-stage1 + weapon-pickup，对 t22（ram-watch-s3.md §8）依赖 f1830+ 帧号的结论逐条重验。

**复核基准 run**（本轮新采，防踩踏三件套生效）：

| 场景 | scenario sha256 | generated lua sha256 | 备注 |
|---|---|---|---|
| gameplay-stage1 | `30563ba1ab01…` | `dba9dec222db…` | 2400 帧；旧产物归档 `analysis/traces/.archive/gameplay-stage1-20261012-155826` |
| weapon-pickup | `8636a34fd4db…` | `32f5c184f8f5…` | 2350 帧；归档 `weapon-pickup-20261012-155917` |

孤儿区（run.json 早于 b80135d）：`exp-shop-*`（13:40-41）、`title-menu`（13:50）、`shop`（14:44）、
t25 三场景（15:23-24）与 t22 时代的 gameplay-stage1（14:44）已被归档/覆盖；本报告不引用其 f1830+ 数据。

---

## 一、复核通过（t22 结论逐条重验，全部 bit 级一致）

| # | t22 结论（帧号均在 f1830+ 危险区） | t22 记录值 | 复核 run 实测 | 判定 |
|---|---|---|---|---|
| R1 | money_counter_bcd 显示值 f2160 | 4400（数字 4,4,0） | `$E0-$E6`=0,0,0,**4,4,0**,0 | ✅ 复核通过 |
| R2 | 同上 f2220 | 5700 | **5,7,0** | ✅ |
| R3 | 同上 f2280 | **6800** | **6,8,0** | ✅ |
| R4 | 同上 f2340/f2349 | 7200 | **7,2,0** / 7,2,0 | ✅ |
| R5 | extend_threshold_bcd = 8000 | `0,0,8,0,0,0` | 五帧全部 `0,0,8,0,0,0` | ✅ |
| R6 | 拾『W』三方同步（f2220） | `$0190` 0→3 + `$018F` 3→4 | probe：f2160=3/0 → **f2220=4/3** → 稳定 4/3 | ✅ |
| R7 | player_x/y 快照（f2160=128/37、f2220+=243/160） | 同左 | watch `player_x/player_y` 逐帧一致 | ✅ |
| R8 | `game_action_mode`（$0401）= 7（gameplay） | 7 | f2220/2280=7 | ✅ |
| R9 | 金钱编码（7 位 BCD 前导零抑制、显示=值本体） | 结构结论 | 复核 run 数字流 440/570/680/720 → 显示 4400/5700/6800/7200 | ✅ 结构再证 |
| R10 | shop 计数器 0 ↔『你有──￥0』（f1080-1440） | 0 | 不在危险区（<f1830），本轮未重跑 shop | ✅ 维持（旧证据合规） |

> 结论：**t22 的金钱计数器/阈值/拾取事件链在确定性驱动下 bit 级复现**，`money_counter_bcd`、
> `extend_threshold_bcd`、`bomb_ammo`、`bomb_type_index` 四条台账（N-0028/0029/0017/0030）维持有效。
> 新观察（非 t22 主张，登记备查）：f2220 同帧 `$0034` 亦 0→3（与 `$0190` 同值）；输入注入存在
> **+1 帧延迟**（schedule 帧 N 的按键在 N+1 反映于 `controller_held`，边界帧差异 12/1091 全部由此解释）。

## 二、更正（证伪更正文化，原地标注不删历史）

### 2.1 W8『游戏内虚拟输入序列』假设 —— 【2026-10-12 证伪（t28）】

- t25 §9.5 观察到 akey-role 迹象：无排程方向位出现在 `controller_held`。
- 复核实验（确定性驱动 + 串行锁）：gameplay-stage1 **1091 帧 0 个外来位**、weapon-pickup **1081 帧 0 个外来位**
  （全部差异 = ±1 帧注入延迟边界帧，无一例计划外按键）。
- **结论**：W8 现象 = **孤儿 run 干扰**（两个 Mesen 进程共写同一输出文件，防踩踏三件套之前的事故形态，
  与 t24 分岔同源），**不是**游戏内虚拟输入序列。t25 的 akey-role f1830+ 衍生观察（如 $0401 窗口 f1916/f2132）
  同属孤儿污染区，N-0036 的窗口时间线证据降级（语义结论 `_todo_401_deploy_semantics` 本就是 C 级假设，维持）。
- 对应修订：`ram-watch-s3.md` §9.5 W8 行原地加注（见该文件）。

### 2.2 N-0032『HUD 字母位=weapon_slot』绑定 —— 【2026-10-12 证伪（t28）】

- 证据反例（确定性 run，像素位图解码）：f2220 拾『W』后 `$0034`=**3**（且 `$0190`=3），但 HUD 行 1 字母
  f2280 仍为 **'A'**（行 1 = `A0`，位图 rows 40-51 复核）→ 字母位 ≠ weapon_slot 直读。
- N-0032 的核心事实（拾取处理 `CMP $34`/`STA $34`、初始化/重置写点）**维持**；但其证据字段中
  『HUD 字母位 0→'A' 对应』绑定**撤回**，字母位来源另考 → 追加 N-0037（`_todo_slot_letter_binding`，C 级）。
- 新疑点备查：f2220 拾取事件同时写 `$0190=3` 与 `$34=3`（同值），`$34` 语义或为『当前弹/武器索引』共用槽，
  待 S2 分化场景（纯数字圈拾取 vs 纯字母圈拾取）复核。

## 三、复现命令

```bash
python3 tools/scenario_run.py run gameplay-stage1 && python3 tools/scenario_run.py run weapon-pickup
# 金钱数字/阈值（zp 流）
python3 - <<'PY'
import json
for line in open('analysis/traces/gameplay-stage1.frames.jsonl'):
    r=json.loads(line)
    if r['f'] in (2160,2220,2280,2340):
        zp=bytes.fromhex(r['w']['zp'])
        print(r['f'], [zp[0xE0+i] for i in range(3,6)], [zp[0xE7+i] for i in range(6)])
PY
# 拾取三方（probe RAM）
python3 -c "
import glob,re
fs={int(re.search(r'_f(\d+)_',p).group(1)):p for p in glob.glob('analysis/traces/gameplay-stage1_f*_probe.ram.hex')}
[print(f, bytes.fromhex(open(fs[f]).read().strip())[0x18F], bytes.fromhex(open(fs[f]).read().strip())[0x190]) for f in (2160,2220,2280)]"
# W8 外来位检查（见报告 §2.1 脚本逻辑：expect(hold 段) vs controller_held 低 4 位）
# 旧产物审计：ls analysis/traces/.archive/
```

## 四、遗留清单

| # | 项 | 级 | 去向 |
|---|---|---|---|
| Q1 | `$34` 与 `$0190` 同写语义（弹/武器共用索引？） | C | S2 分化拾取场景 |
| Q2 | HUD 字母位来源（letter-bank 或查表） | C | S2/S3 |
| Q3 | N-0036 的 $0401 窗口时间线需在干净 run 重采（akey-role 建议列入防踩踏后重跑清单） | C | 下批 S2 |
| Q4 | 注入 +1 帧延迟约定写入 oracle/scenarios/README（差分比对需按此对齐） | B | 建议 t12/t24 owner 补文档 |
