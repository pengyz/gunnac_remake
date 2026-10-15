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
