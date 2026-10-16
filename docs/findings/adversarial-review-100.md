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
