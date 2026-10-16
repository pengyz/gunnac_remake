# audio-golden.md —— L-Audio 门禁层设计（Stage A 音频 golden，P2 spec）

> 状态：2026-10-16 v1 设计稿（lead 三裁定定案：60f 帧窗/DMC 写流一体/无需速率归一化）。
> 方案=A 案寄存器写流 hash（[audio-domain-survey.md](../findings/audio-domain-survey.md) §3）。
> 实现归宿：game-oracle 差分管线 L-Audio 层（与 L1-L4 并列）。

## 1. 对拍对象（确定性事件流）

**APU 寄存器写流** = 游戏逻辑对 $4000-$4017 的全部写事件（帧索引三元组）：

```
event := (frame: u32, reg: u8, value: u8)    ; reg ∈ $00-$17（$4000+reg），写序保真
```

- **DMC 一体**：$4010-$4015 写事件入流（lead 裁定②）；**样本内容**（$4012/13 指向的
  采样字节）不入 hash（Stage B 合成器后补样本内容维）。
- 帧索引=游戏逻辑帧（60Hz tick），非音频采样率——**无需速率归一化**（裁定③）。

## 2. 分段与指纹（60f 帧窗定长，裁定①）

```
segment k := 事件流中 frame ∈ [60k, 60k+60) 的子序列（有序、含写序）
seg_hash(k) := SHA-256( canonical_encode(segment k) )
golden := JSONL { "seg": k, "scene": <场景 id>, "hash": <hex>, "events": <n> }
```

- canonical_encode：`frame_delta(varint) | reg | value` 逐事件紧凑编码（帧内相对段首）；
  空段=hash(空串)（显式记录，非缺失）。
- **场景对照**：golden 行带 scene 字段（与 s5/l5 台账 scene id 同源），段号全局递增——
  同 scene 段可跨 run 对拍。

## 3. L-Audio 门禁层设计

| 面 | 口径 |
|---|---|
| 采集 | game-oracle 回放输入时钩取 APU 写流（Mesen2 内存回调 $4000-$4017 写监视 → 事件流） |
| golden 生成 | `tools/audio_golden.py gen <trace>` → `data/golden/audio/<run>.audiogold.jsonl` |
| 对拍 | `tools/audio_golden.py diff <a> <b>` → 逐段 hash 比对；差异段报事件级 diff（首差异三元组） |
| 门禁 | `gates/cli.py` 增 L-Audio 段（与 trace/visual 并列）；基线 ratchet 同机制（只增不降） |
| 容忍 | **0**（保真）；差异段自动导出双方事件流片段供定位 |
| 演进 | Stage A=写流 hash；Stage B=+样本内容 hash（合成器落地后，DMC 内容维） |

## 4. 与差分管线的接缝

- L1（子程序）/L2/L3（帧 trace）/L4（视觉）/**L-Audio（写流）**——audio 与 trace 可同源自
  Mesen2 run（一次回放双产物：RAM/trace + APU 写流）。
- 事件流与 [s5-all-clear.anchors.jsonl](../../oracle/traces/s5-all-clear.anchors.jsonl) 的
  scene/f 字段共享口径（音频段可锚到帧锚表）。

## 5. open items（实现前待定）

1. canonical varint 细则（u32 帧差编码格式冻结）。
2. Mesen2 写监视回调挂点（A10 沙盒内）——AUD-1 驱动入口定谳后一并定。
3. golden 目录命名（`data/golden/audio/` 基线登记进 assets-manifest 机制）。
