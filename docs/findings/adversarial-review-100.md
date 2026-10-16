# 对抗复核 #100 — 近批复核债（音频三件套+R-4 屏流提取器+回归修+R-4b+八形谱）

> 日期：2026-10-16｜角色：adversarial-reviewer（证伪立场；实施方=多棒≠审核方）
> 被审：8931f92 APU 写流+规范编码+60f 分段 / 1b1e6ca audio_golden.py / bank11 驱动 /
> 7151516 R-4 屏流提取器 / db4c9c1 stg02-start 回归修 / f209330 R-4b 命名面 / 批3 bank7 八形谱。
> 四证伪靶：①audio hash canonical encode 同式核 ②屏流提取器链式拼接（$A129+$A13D=45B）字节核
> ③八形谱抽 2 ROM 重导 ④回归修 fallback 语义。拆段落盘。

## 段 A：①audio hash canonical encode 与 game-core 同式核——**API 边界同式成立** ✅

### A.1 逐要素同式对账

| 要素 | game-core（lib.rs） | audio_golden.py | 裁定 |
|---|---|---|---|
| varint（LEB128） | `byte=v&0x7F; v>>=7; …byte\|0x80` | 同构循环 | ✓✓ **算法逐字节等价** |
| frame delta | `e.frame.saturating_sub(base)` | `max(0, frame-base)` | ✓ 等价 |
| reg | `write()` 入口 `reg&0x1F`（结构文档=$00-$17） | 编码侧 `reg&0x1F` | ✓ **API 边界等价**（掩码时点不同——见注） |
| value | `push(value)`（u8） | `value&0xFF`（等价） | ✓ |
| 空段 | 空字节串（"hash(空串)显式记录非缺失"注） | `b""`+自检断言 | ✓ 语义一致 |

### A.2 样例手算交叉（独立验证）
`[(100,15,3F),(160,00,08),(161,17,FF)]` ⇒ `00 15 3f | 3c 00 08 | 3d 17 ff`
（varint(0)=00/varint(60)=3C/varint(61)=3D 逐段手算）——与 `canonical_encode` 实出
**逐字节全等** ✓；`--selftest ok` ✓（稳定性+空段断言）。

**注（轻量）**：reg 掩码时点不对称（Rust=write() 入口掩/Python=编码时掩）——经 API 路径等价，
但 ApuWrite 公有结构直构可绕过 write() 掩码 ⇒ 建议 `canonical_encode_segment` 内再掩一次
（防御性对称）；"格式冻结=open item ①"（本实现=候选冻结面）注**诚实** ✓ golden 兼容性
以最终冻结为准的口径在案。

---

## 段 B：②屏流链式拼接字节核 + ③八形谱抽 2——**全等** ✅

### B.1 ② 屏流提取器链式拼接（$A129+$A13D）——**45B=20+25 精确 + 20B 重叠逐字节同** ✅

ROM 直读（bank1 窗）：
- `$A129` 首屏记录（至 FF @offset19）= **20B** `e3 21 e6 e7 e8 fe 03 22 f6 f7 f8 2d×7 24 ff`；
- `$A13D` 解锁屏记录（至 FF @offset24）= **25B** `23 22 a0 a1 a2 86 87 20 8b 8c 8d fe 43 22 90 91 92 96 97 2c 9b 9c 9d 21 ff`；
- **链式续段注实证**：$A129 offset20 起 20B == $A13D 头 20B **逐字节相同** ✓✓
  （自算重叠 20B 与"尾段与头段逐字节同"注精确一致）；
- **拼接总长 = 20+25 = 45B** ✓✓ 与声称"$A129+$A13D=45B"精确闭合（重叠去重口径）。

### B.2 ③ 八形谱抽 2 ROM 重导（批3 bank7）——**双形态逐指令精确** ✅

| 形态 | ROM 直读（bank7 窗） | 声称 | 裁定 |
|---|---|---|---|
| **⑦ $A61D**（$4A8 索引速度合成） | `BC A8 04`(**LDY $04A8,X**)`B9 DC A6`(**LDA $A6DC,Y**)`18 7D E0 04`(**CLC/ADC $04E0,X**)`29`（AND #$F0 起手） | "LDY $04A8,X/$A6DC,Y→ADC $04E0/AND #$F0/ORA #$08 位域合成" | ✓✓ 逐指令精确 |
| **⑧ $A6EC**（子对象生成器） | `20 04 C3`(**JSR $C304** 槽分配)`B0 1E`(BCS 失败支)`BD 29 AC 99 E0 04`/`BD 31 AC 99 FC 04`（速度对装） | "JSR $C304 槽分配→type 写+速度/参数装" | ✓✓ 精确 |

形态谱六→八形（⑦$4A8 索引合成/直装 ⑧子对象生成器）双新形立证 ✓；
枚举口径注（"分发表 bank_switch=$07 唯一目标 12（lead 口径 13——差 1 疑共享入口/计数口径，
照录待核）"）**诚实** ✓（差异挂账不抹平）。
