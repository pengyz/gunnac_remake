# 对抗复核 #88 — 批次 4a：wave_record.rs（CA 判定链/sub_8DBD/BUG-021/$D86B 表）

> 日期：2026-10-15｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：commit `b1d96bf`（wave_record.rs + tests/wave_record.rs 6 用例 + #87 advisory 顺带清）。
> 五证伪靶：①bit4&5 组合判（AND#$30==#$30≠OR）②分派≠执行（$CB bit1 门）③bit5 零效果锁死（D4 保真）
> ④$D86B 256 项字节精确+6 锚 ⑤sub_8DBD 真值表四组合。纪律：mutation 即做即还原；拆段落盘。

## 段 A：④$D86B 状态转移表独立重导——**256/256 字节全等** ✅

### A.1 方法（独立性）
ROM 直读 `rom/gunnac_cn.nes` 偏移 `0x10+14×0x2000+(0xD86B&0x1FFF)=0x1D87B` 起 256 字节
（自算偏移，不调被审测试代码），与 `wave_record.rs` 的 `STATE_TRANSITION[256]` 逐项比对。

### A.2 结果

| 项 | 自算 | 裁定 |
|---|---|---|
| 表项数 | 256 | ✅ |
| 字节精确 | **256/256 全等**（ROM 表 sha256 前缀 `09e24e74…`） | ✅ |
| 6 锚 | $01→$0F ✓ / $22→$12 ✓ / **$4A→$4A 恒等** ✓ / $5F→$17 ✓ / $02→$02 ✓ / $12→$12 ✓ | **6/6 精确** |
| 读点 | listing `bank14:D619 LDA $D86B,Y`、`bank14:D65C LDA $D86B,X` 双读点在案 | ✅（与声称 $D619/$D65C 一致） |
| 负向哨兵面 | 恒等项实测 **51/256**（非全恒等）⇒ "identity 映射必红"断言有真实咬合面 | ✅ |

**④ 成立**：实现表=ROM 字节忠实镜像，锚点语义（玩家受击→dead/召唤物相位切换/中 boss 恒等）与
arena-midboss §2.2 实测转移一致（锚点出处 B 级交叉，字节面 A 级）。
