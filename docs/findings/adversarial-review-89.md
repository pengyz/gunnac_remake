# 对抗复核 #89 — 批次 4c：运动机三表/mod-8 步进/字段复用/型值迁移（17eb4b5）

> 日期：2026-10-15｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：commit `17eb4b5`（enemy.rs 三表+motion_step+生命周期注+TYPE_MIGRATION_SLOT09；tests ⑫⑬）。
> 五证伪靶：①三表 ROM 直读 ②$534 EOR 仅吃 $2B bit0 ③两段生命周期锁 ④诚实边界 ⑤对拍 171/121/12。
> 纪律：mutation 逆串精确还原（禁 git checkout——gotcha-shared-tree-mutation-restore）；拆段落盘。

## 段 A：①三表 ROM 直读对拍——**三表逐字节全等** ✅

| 表 | 声称 | ROM 直读（off=0x10+5×0x2000+（addr&0x1FFF）自算） | 实现常量 | 裁定 |
|---|---|---|---|---|
| TRI_WAVE_684 @$AC5F | 对称三角波 D3 D7 DB DF E3 DF DB D7（B1 §3/#76 F2 真值） | `d3d7dbdfe3dfdbd7` | 同 | ✅ |
| TRI_WAVE_534 @$AC67 | 逐字节 | `0343434303030303` | 同 | ✅ |
| TRI_WAVE_518 @$AC6F | 逐字节 | `2424242421242424` | 同 | ✅ |

**双寻址交叉**：bank5 列表窗（$8000-$9FFF）同字节现于 `bank05:8C5F/8C67/8C6F`（data-as-code
误译为 DCP/SLO/BIT 指令行但字节序列一致）⇒ "$AC5F"=bank5 挂 $A000 窗（R7）时的视图，
两寻址同源 ✓。三角波对称性（D3…E3…D7）与声称真值逐项一致 ✓。
**① 成立**（三表=ROM 字节忠实镜像，24/24 字节）。

---

## 段 B：②bit0 判据 + ③生命周期锁 mutation——**双咬 ✓ + F-C89 盲区**

### B.1 mutation 批实测（逆串精确还原，基线 7/7 绿）

| 突变 | 靶 | 实测 | 裁定 |
|---|---|---|---|
| M2a 全字节 EOR（去 `&1` 掩码） | ② | ✅ 1 红（`motion_phase_bit0_and_field_reuse`） | 咬 |
| M2b bit1 替 bit0（`&2`） | ②（lead："喂 bit1 应不受影响/受了必红"） | ✅ 1 红 | 咬 |
| M3 字段混用（v534 改用 518 表） | ③ 两段生命周期锁 | ✅ **2 红**（⑬+⑫） | 咬 |
| M-wrap 回绕破坏（wrapping→saturating） | 8-bit 回绕 | ✅ 1 红 | 咬 |
| **M-mod8 索引掩码 `&7`→`&3`** | mod-8 步进 | ⚠️ **7 测全绿** | **盲区（F-C89）** |

**② 成立**（bit0 判据双向咬合：全字节 EOR 与 bit1 替换皆红）；**③ 成立**（混用即红，2 测咬住）。

### B.2 F-C89（必修，1 行级）：mod-8 索引无哨兵可咬

**实现正确**（listing `AND #$07`，见段 C 逐指令）——但⑫ 步进 golden 的 idx 采样**只有 0 与 1**
（counter=0→idx1 / counter=7→next8→idx0 / counter=255→next0→idx0）：
`&7→&3`（甚至 `&1`）在 idx∈{0,1} 上恒等 ⇒ 突变全绿。"mod-8 回绕"断言实际测的是 **counter 回绕**，
不是掩码宽度——**测试名与覆盖面错位一格**（与 #84 F-B4/#88 F-C88 同类第三例）。

**修正建议**：补 idx≥4 golden 一例（如 `motion_step(0x04,0x00)` 断言
`(v684,v534,v518)=(0xDF,0x03^?,0x21)` 即 idx5 三表值；或 counter=6→idx7 断言 D7），
`&7→&3` 必红。表常量本身已被① 24 字节 ROM 比对锁死，此为**步进掩码面**补咬。
