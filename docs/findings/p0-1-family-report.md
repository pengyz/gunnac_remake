# P0-1 差异族攻坚·族①（nametable 内容空/上传几何）首迭代报告

> 【2026-10-20 00:55】实施=Lead（mimo；实施子 agent 后台/前台 3 连崩，按纪律"结果优先"由 Lead
> 实施、独立 mimo 子 agent 复核把关，已落库披露）。**本迭代结论=混合负结果**，如实入账。

## 1. 族①现状（attack-plan §3.1，已复核口径）

投影/nametable 内容空 = **209,086 px（74.63%）**，主因定位（A 级）：默认路（env={}，即签署 actual
来源路）的 NMI 上传几何与 A 级模型不符——`render_frame.rs` 旧实现为**单行 32 连写**（base=hi:lo），
而 ROM 定谳模型（bank15:$F040-F0DA；game-core `terrain.rs::nmi_flush` 镜像）为
**base=($8F:$8E)+$0F、4 组×8B 组间步进 $20、attr 行=(($8F|3):$90)+$0F 8B**；且 attr 旧路被忽略。

## 2. 本迭代改动（已入库，模型路 env 门控）

- `pending_row` 增几何模式位；**默认=legacy 32 连写（行为与基线逐字节一致，回归核 2352/3742 全同）**；
- `MODELGEO=1` 启用 A 级模型镜像（4×8 分组 +$0F/+$20 + attr 行）；HUD LUT 路保持 legacy 形态；
- FLAGUPLOAD=1 经验拟合路未动（attack-plan E1 约束）；测试 52 项全绿。

## 3. A/B 实测（同 binary/同输入，唯一变量=MODELGEO）

| 对 | before（legacy） | after（MODELGEO=1） | Δ |
|---|---:|---:|---:|
| l4_stg02-start（gs5-anchor-render f3450） | 2,352 | **2,138** | **−214** |
| gameplay-stage1_f01260 | 3,742 | 4,054 | **+312** |
| l4_clear-inject（stg03 f4600） | 58,159 | 58,169 | +10 |

**混合，非单调改进**：#14 改善 214px，gameplay 恶化 312px，clear-inject 持平。

## 4. 结论（纪律判定）

严格按 A 级模型镜像**未胜出** ⇒ 说明模型应用映射仍有未解交互项，已知嫌疑（均有文档线索）：
1. **行提取窗**：buf=$0130-$0160（0x31B）中哪 32B 是"行缓冲 $130-$14F"（旧路 ROWWIN 可调、经验值 0）；
2. **四字段映射**（terrain.rs 注 t93 ③）："记录 4 字段 → 行内列 (x, x+1, x+16, x+17)"——32B 可能不是
   简单 32 tile 行序，而是 4 记录×字段展开；
3. **attr 相位**：attr 行地址/内容时点（$D3-$DA 双组×7B vs 8B）；
4. **$0160 批次语义**：多批/帧时单快照丢行（旧注 4020 写/帧批）与 v=1/v=3 脉冲语义（FLAGUPLOAD 路注）。

按 AGENTS §3.5 **禁止试错式改机制**：不再盲试几何变体；**回退默认=legacy**（基线可比、管线稳定），
模型路保留 `MODELGEO=1` 供取证后迭代。

## 5. 下一步（取证卡建议，优先级序）

1. **行缓冲字段映射定谳**（t93 ③ 升 A）：桥写集 trace 对照 $130-$14F 字节→屏幕列序——解开 (x,x+1,x+16,x+17)；
2. **$0160 批次语义**（v=1/v=2/v=3 三件套）：probe 实测（gameplay 550 buf 帧/107 flag 非零帧/帧内 4000+ 写脉冲）
   与 ROM 12 写点族对账——决定单快照 vs 批次队列；
3. 完成 1+2 后再实施族①第二迭代（预期接通后 ~200K px 量级下降）。

## 6. 复现命令

```
cargo test -p game-desktop                       # 52 项全绿
target/p0-0/build/debug/render-frame.exe 3450 stg02-start gs5-anchor-render --out-dir target/p0-0/render/p01-run1 --log-path target/p0-0/render/p01-run1/r1.log
# MODELGEO=1 同命令另渲 → diff_visual.compare 计 len(samples)
```
