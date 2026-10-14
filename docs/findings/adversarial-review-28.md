# 对抗复审 #22（t197）：t193 八关骨架 + R18-1 闭合复审

> 被审方：t193（随 2ef86a1 入库 + 3170fe2 基线）。审核方：verifier（t197），ROM 目录独立解析 + 测试全量复跑。
> **裁定：pass（R18-1 闭合 + 八关骨架对拍全中）**——①②③④⑤全过。观察 1 条（琐碎）。

## 0. 观察（琐碎）

- O1：`stg_02_08_chain_checkpoints` 的 root/tail 非零哨兵为弱负向（可加"越界指针必拒"）；f=0 占位行
  过 l5_legality（帧序单调）——"fail-closed"语义=不发明帧号 ✓（占位待取证填真），与负向模式校验分层清晰。

## 1. ①STG-02..08 骨架装载复核 —— ✓✓ ROM 独立对拍全中

**独立解析**（bank8 目录 8B/行：root|pattern|params|tail，按 t84 语义）：

| 关 | root | tail | t193 断言 |
|---|---|---|---|
| STG-2 | $A048 | $A3C4 | (0xA048, 0xA3C4) ✓ 逐值 |
| STG-7 | **$A000** | $A5D6 | == STG-1 root ✓（共享流） |

- 8 行全解析（STG-1..8 root/tail 全出）；t87 §1 逐值断言（rec4/rec6/loop_n、tick_params 7 段、scripts 8）
  随 `stg_02_08_skeleton_registry`/`stg_02_08_rom_root_tail_verified` 测试过 ✓；
- **cargo test 103/103 独立复跑** ✓（基线 98→103 一致）；STG-9/0 → None fail-closed ✓。

## 2. ②链扩展复核 —— ✓

- golden 扩展 7 行（stg02-08-chain，每关 StageStart 检查点）→ 口径测试更新 **4 场景/17 事件/15 命名** ✓；
- **f=0 占位 fail-closed** ✓："帧号=占位（场景取证后填真值，不发明）"——与 AGENTS"未证不发明"一致；
- root/tail 非零哨兵负向 ✓。

## 3. ③R18-1 闭合复核 —— ✓ 双层调和成立

- **p0 真值口径实装+负向**：`ink_mask_p0_set_negative_highplane`（1/3=墨、0/2=底；高平面 v==2 口径矛盾必红哨兵）
  ——引本审 t185 实证"oracle『地』ink=p0 掩码 100%（256/256）"入断言 ✓ **R18-1 闭合**；
- **t192 裁定对账（两说相容）核** ✓：文本块 "index2→白"=该块 palette **具体表值**（非全局 index 规则）——
  字模族 tile 两半重复（p0==p1，值仅 0/3）⇒ 文本块 index2 映射=抑制绘制（帧上文本行本就近空白），
  与 R18-1（CJK 精灵面墨=p0 置位）**分层相容** ✓；"勿固定 index"裁定与负向哨兵不矛盾（不同作用域）。

## 4. ④index race 归属注核 —— ✓

3d41ac4 归属记录在（"t192 commit 含 t193 工作——并行竞态注，归属记录保留"）✓ 竞态透明、归属未混。

## 5. ⑤商店购买/GAME OVER fail-closed 维持核 —— ✓

GAME OVER/CONTINUE/商店购买（未证模式）由 l5_legality 未知模式 fail-closed + flow.rs 已证边集拒止双层覆盖
（测试在：`GameOverUnproven` 必拒、"GAME OVER/CONTINUE 未证段"注）✓ 维持。

## 6. 复现命令

```bash
cargo test --workspace 2>&1 | grep -c "test result: ok"   # 103 passed
python3 - <<'PY'   # ① 目录独立解析
rom=open('rom/gunnac_cn.nes','rb').read(); b8=rom[0x10+8*0x2000:0x10+9*0x2000]
for i in range(8):
    b=b8[i*8:i*8+8]
    print(i+1, hex(b[0]|b[1]<<8), hex(b[6]|b[7]<<8))
PY
grep -n "f=0\|占位" crates/game-oracle/tests/l5_chain.rs   # ②
```
