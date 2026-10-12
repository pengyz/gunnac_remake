# W-NEW 行缓冲帧内时序取证（t94）— NMI 级插桩 + 时序模型定谳

> 方法：静态所有权反查（seed）+ **NMI 级写点插桩**（`probe_mem` 观察性扩展，任务卡授权）。
> L4 内容帧断言面 = §3；t93 快照悖论 §2.3 动态解释。

## 1. 插桩落地（工具侧观察性扩展）

- `tools/scenario_run.py`：新增 `probe_mem <lo> <hi>` 指令（PLAN.probe_mem）。
- `tools/scenario_capture.lua`：PLAN.probe_mem 存在时注册 **内存写回调**（Mesen2
  `emu.addMemoryCallback`，pcall 双拼写兼容），逐事件落 `analysis/traces/<name>.probe.jsonl`
  （帧号/PC/地址/值，20 万条上限）。
- `oracle/scenarios/fire-hold.txt`：追加 `probe_mem 0130 0160` / `probe_mem 00d3 00da`。
- 采量：**196,738 写事件**（fire-hold 2350 帧）；回调注册双区间成功（cb:true ×2）。
- 限制：**回调内 `emu.getState` 不可得 PC**（返回 ????）——相位证据改用**写序时序**（§2）。

## 2. 时序模型定谳（A 级）

### 2.1 静态所有权（seed 反查）

| 部件 | 读写 | 位置 |
|---|---|---|
| **行缓冲 $130-14F（32B）** | 写：bridge（主循环） | bank00:$85D0/$85DC（t64 桥） |
| | 读：**NMI 上传**（vblank） | bank15:$F09A-$F0D0：`$130,X→$2007` ×8，X+=8 至 $20（4 组×8B）；目标地址 = **$02 累加（+$20/组）** |
| **挂起 $160** | 写：子系统请求（STA，值=类型） | b0:$827A/$88A3/$900A、b2:$8DA6、b3:$BA3B-$BBBF、b5:$853F |
| | 清：**NMI 上传后 `STX $160`（X=0）** | bank15:$F14B（调色板路径尾） |
| **调色板源 $161-$167** | 读：NMI 调色板上传（$3F00/$3F10 目标，2 组×7B） | bank15:$F105-$F13B |

### 2.2 动态写序相位（probe 实测）

- **帧内双相**：`f6` 型 = **升序扫**（$130→$143…整缓冲填入，459 事件）；`f5` 型 = **降序扫**
  （$15F→$130，465 事件，启动期）——bridge 填充为**整块扫写**。
- **$160 写值分布：{0: 2321, 3: 149,379}**——3=调色板挂起类型（NMI `CMP #3 → $F105` 路径）；
  高频写 = 子系统逐帧请求（观察值，写者语义维持 t93 判定）。
- 行缓冲写覆盖 **532/2350 帧**（23%）——其余帧无内容（L4 内容帧稀疏性的动态注记）。

### 2.3 t93 快照悖论解释（W-NEW 收口）

endFrame 快照见『$130-14F 恒 0 与 $160≠0 无交集』= **读空窗口相位**：NMI 于 vblank 读走
缓冲（读后即废，无清零）→ endFrame 落点恰在『NMI 已读、bridge 未再写』区间；$160 的清零
仅发生在调色板路径尾（STX）→ 可见 $160≠0 而缓冲空。**非数据矛盾，是采样相位** ✓。

## 3. L4 内容帧断言面（报 captain，登记归 spec-writer，移植归 porter）

**R-l4-linebuf 时序不变量（机器可判定，A 级）**：
1. bridge 填充 = 整块扫写（升序 $130→$14F，32B；启动期可降序）；
2. NMI vblank 上传 = `$130+$02 段 → $2007`，4 组×8B，地址步进 +$20；
3. `$160` = 挂起类型（0=无、3=调色板），**NMI 上传后清零**（谁写谁清闭环）；
4. 调色板上传源 = $161-$167（$3F00/$3F10 双组×7B）；
5. 断言采样口径 = **NMI 级**（endFrame 快照不足——相位陷阱 §2.3）。

## 4. 复现命令

```bash
python3 tools/scenario_run.py run fire-hold     # probe_mem 插桩自动生效
python3 - <<'PY'
import json
from collections import Counter
evs=[json.loads(l) for l in open('analysis/traces/fire-hold.probe.jsonl') if '"a"' in l]
print(Counter(e['a'] for e in evs).most_common(8))
print(Counter(e['v'] for e in evs if e['a']=='0160'))
PY
sed -n '/bank15:F09A/,/bank15:F14E/p' reference/seed/gunnac.asm
```

## 5. 遗留

| # | 项 | 级 |
|---|---|---|
| W33 | 回调内 PC 轨迹不可得（Mesen2 getState 限制）→ 子帧 PC 级证据待 API/替代法 | C |
| W34 | $160 各写者语义（b0/b2/b3/b5 五族请求类型） | B |
| W35 | $D3-$DA 位属性上传支路（probe 采得 5,577 写，路径未追） | C |
