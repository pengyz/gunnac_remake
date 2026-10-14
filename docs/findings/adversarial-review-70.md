# 对抗复核 #70：DEV-2 修复效果复审（第二轮轻量，AR69-1 修复批）

> 复核人：独立对抗复核员（fresh context，证伪立场；非实施方）。
> 被审：`7d7a964`（`fix(core): AR69-1 两写点净效应落地（$26&=$80）+ 复核 #69 归档`）——
> 对前审 [adversarial-review-69.md](adversarial-review-69.md) 唯一必修 **AR69-1（两写点净效应）** 的修复。
> 方法：不采信被审方/commit 自述——ROM 字节独立直读两写点+执行序、代码/测试逐行核对、
> spec/findings 残留 grep 全仓、对拍三件+双版独立复跑。证据分级依 AGENTS.md §3。
> **Verdict: needs_revision（AR69-1 行为实质全落地且 ROM 复核再确认；唯 1 项文档收口缺口 AR70-1）**

## 0. Verdict 摘要

AR69-1 的**行为实质**（两写点净效应 `$26&=$80`，bit4 终态=0）已完全落地，并经本复核独立 ROM 直读证实：

- flow.rs 双写点镜像（E0F4 瞬态 → E1BB 终态 `&= 0x80`）✓
- f12 断言改净效应模型（bit4==0；bit3==0 保持）✓
- spec R-continue-reset 两写点+净效应表述 ✓
- ROM 直读 `26 F7`/`26 80` 族 + 线性必达执行序 ⇒ 净效应 `$26&=$80` 裁定**再确认** ✓
- 对拍全绿：cargo **135/135** + l2 CN **121/121** + JP **121/121** + gates **12/12** ✓

**但 commit message 自述"A2 §3.2 原地 dated 更正（E1BB 缺口转已裁定）"不成立**：
`docs/findings/game-over-continue.md`（=A2，spec 抬头自称的"证据基准"）**未被 `7d7a964` 触及**
（`git show --name-only` + `git log` 双证，见 §2.3），其 §3.2 复位范围表与 §2 `$26` 行均**无 dated 更正**、
全文 `E1BB` **零命中**；且旧"缺口/未接 E1BB"模型残留于 spec:141/158/176 + flow.rs:485/488，
与已实现的净效应 `&= 0x80` **自相矛盾**。依 AGENTS.md §3.6「证伪更正文化」，被推翻的结论须在原文
原地标注日期更正、不删历史主张——此 dated 更正未落。⇒ **needs_revision**（唯 1 项、纯文档、**零行为影响**）。

## 1. 六项核验表

| # | 核验项 | 裁定 | 依据 |
|---|---|---|---|
| 1 | flow.rs `apply_continue_reset` 两写点镜像（E0F4 瞬态 `(...&0xF7)\|0x10` → E1BB 终态 `&=0x80`，净 bit4=0） | **通过** | §2.1 |
| 2 | f12 断言 bit4 **==0**（旧"置位==0x10"已改净效应）+ bit3==0 保持 | **通过** | §2.2 |
| 3 | spec R-continue-reset 两写点+净效应 **且** A2 findings §3.2 dated 更正（E1BB 缺口转已裁定） | **部分**（3a ✓／3b ✗） | §2.3 |
| 4 | 独立 ROM 直读 E0F4-E0FA / E1BB-E1BF 两写点字节 + 线性后继无跳过执行序，净效应 `$26&=$80` | **通过** | §2.4 |
| 5 | grep 全仓无残留"bit4 置位"旧模型**断言** | **通过**（硬断言清净）｜软残留注释并入 AR70-1 | §2.5 |
| 6 | AD69-2 挂注（R-G4 并入）在文 | **通过** | §2.6 |

**判据**："全落地即 pass"。第 3 项 3b 未落地 ⇒ 非全落地 ⇒ **needs_revision**。

## 2. 逐项核验

### 2.1 核验 1：flow.rs 两写点镜像 —— 通过（A 级）

`crates/game-core/src/flow.rs` `apply_continue_reset`（L492-494）逐行直读：

```rust
fn apply_continue_reset(st: &mut DeathChainState) {
    st.lives = 2; // E0EF
    st.revive_flags = (st.revive_flags & 0xF7) | 0x10; // E0F4-E0F8 瞬态写（先行）
    st.revive_flags &= 0x80; // E1BB-E1BF 终态写（后继必达——复核 #69 裁定：两写点净效应 $26&=$80，bit4=0）
```

- 第一写点（瞬态）`(... & 0xF7) | 0x10`：bit3 清、bit4 置 1 ✓（对应 ROM `29 F7 09 10`）
- 第二写点（终态）`&= 0x80`：只留 bit7 ✓（对应 ROM `29 80`）
- **净效应手算**：`((X & 0xF7) | 0x10) & 0x80 = (X & 0x80) | (0x10 & 0x80) = X & 0x80` ⇒ bit4=0、bit3=0、bit7 保留。
  两写点镜像成立，顺序（瞬态先行→终态后继）与 ROM 执行序一致（§2.4）。

### 2.2 核验 2：f12 断言净效应 —— 通过（A 级）

`crates/game-core/tests/death_chain.rs` `f12_continue_reset_set`（L233-234）逐行直读：

```rust
assert_eq!(st.revive_flags & 0x08, 0, "$26 bit3 清零=复活权恢复（E0F4 瞬态写）");
assert_eq!(st.revive_flags & 0x10, 0, "$26 bit4 终态=0（复核 #69 净效应 $26&=$80：E0F4 瞬态→E1BB 终态两写点序）");
```

- bit4 断言 `& 0x10 == 0`：**净效应模型**（旧"置位 `== 0x10`"已改）✓
- bit3 断言 `& 0x08 == 0`：**保持** ✓
- 全仓 grep `revive_flags`：无任何断言仍持 `& 0x10 == 0x10`（旧置位模型）✓（详见 §2.5）

### 2.3 核验 3：spec 两写点表述 ✓ ／ A2 findings §3.2 dated 更正 ✗ —— 部分（3a 通过，3b 缺失）

**3a 通过**：`docs/spec/game-over-continue.md:135` R-continue-reset 已含两写点+净效应：

> `$26 两写点序（E0F4 瞬态 `($26&$F7)|$10` → E1BB 终态 `$26&=$80`）**净效应 $26&=$80**`

**3b 缺失（核心）**：commit message 称"A2 §3.2 原地 dated 更正（E1BB 缺口转已裁定）"，但**未落**：

| 复核动作 | 实测结果 |
|---|---|
| `git show 7d7a964 --name-only` | 改动文件=`flow.rs`/`death_chain.rs`/`adversarial-review-69.md`/`spec/game-over-continue.md` —— **不含** `docs/findings/game-over-continue.md`（A2） |
| `git log --oneline -5 -- docs/findings/game-over-continue.md` | 末次改动 `8bee3c6`（在 `7d7a964` 之前）——A2 未被本修复批触及 |
| `grep -c E1BB docs/findings/game-over-continue.md` | **0** —— A2 全文无 E1BB，无从"缺口转已裁定" |
| A2 §3.2 复位范围表（L253-263）逐行读 | **无 `$26`/E1BB 行**，无 dated 更正 |
| A2 §2 `$26` 行（L153）逐行读 | 仍只记瞬态写 `E0F4-E0F8: LDA $26 / AND #$F7 / ORA #$10 / STA $26`，**无 `【…更正（复核 #69）】`** |
| A2 全文 `更正` marker grep | 有大量 `【2026-10-14 更正（复核 #63 AR63-1）】` 等既往 dated 更正（自证本文档的更正规范），**唯 #69 无一条** |

⇒ **A2 findings §3.2/§2 的 `$26`/E1BB 结论被 #69 证伪，却未按 AGENTS.md §3.6 原地 dated 更正**。
commit message 的"A2 §3.2 原地 dated 更正"= **overclaim（自述与 diff 不符）**——本复核证伪立场下的主要发现。

**附带（同根残留，旧"缺口/未接"模型与实现矛盾）**：`E1BB 从缺口转已裁定` 的销账未做完，旧模型残留于：

| 位置 | 残留文本 | 矛盾点 |
|---|---|---|
| `docs/spec/game-over-continue.md:141` | 注 1"…为 A2 §3.2 未列写点——**规格缺口**…本实现按 A2 复位集闭合，**未接 E1BB**" | 与 :135 净效应、与 flow.rs:494 `&=0x80`（已接 E1BB）矛盾 |
| `docs/spec/game-over-continue.md:158` | "**E1BB 缺口**：…"（§2 负结果清单） | AR69 §2.4 pt2 要求"从 §2 遗留缺口清单销账"，未销 |
| `docs/spec/game-over-continue.md:176` | "遗留假设: …E1BB **规格缺口**（上报 spec-writer）" | 同上，未销 |
| `crates/game-core/src/flow.rs:485` | 复位集摘要行仍列 `$26:=($26&$F7)\|$10`（该行未标"瞬态"） | 摘要单写模型，与 :493-494 两写点净效应不一致（注 1 可补救但该行孤立表述误导） |
| `crates/game-core/src/flow.rs:488` | 孤立残句"…原缺口记录如下（历史）／spec-writer）——本实现按 A2 复位集闭合，**未接 E1BB**。" | 断句残缺（"（历史）"后接"spec-writer）"），且"未接 E1BB"与 :494 `&=0x80` **直接矛盾** |

### 2.4 核验 4：独立 ROM 直读两写点 + 执行序 —— 通过（A 级，裁定再确认）

不采信前审/被审字节引用，独立读 `rom/gunnac_cn.nes`（PRG 偏移 `0x1E000 + (cpu-0xE000)`，bank15）：

```
第一写点 E0F4-E0FA:  a5 26 29 f7 09 10 85 26
  E0F4 A5 26   LDA $26
  E0F6 29 F7   AND #$F7    ; bit3 清
  E0F8 09 10   ORA #$10    ; bit4 瞬置 1
  E0FA 85 26   STA $26     ; = 瞬态写

第二写点 E1BB-E1BF:  a5 26 29 80 85 26
  E1BB A5 26   LDA $26
  E1BD 29 80   AND #$80    ; 只留 bit7
  E1BF 85 26   STA $26     ; = 终态写

执行序上下文 E1B0-E1C7:
  E1B0 20 1d 83 20 1c 92 a9 01 8d 00 04 | a5 26 29 80 85 26 | a9 00 85 27 85 16 85
       …  E1B6 LDA#1/STA $0400  │  E1BB 终态写        │  E1C1 LDA#0/STA $27/$16…
  控制流字节（JMP 4C / Bxx D0 F0 10 30 90 B0）于 E1B6..E1D1 区间 = NONE（线性 fall-through）
```

- 两写点字节 = `26 F7`/`26 80` 族 ✓，逐指令对上。
- **执行序**：E1B6（`$0400:=1`）→ E1BB（终态写）→ E1C1（`$27:=0`）**线性后继、区间内零控制流指令、无分支跳过** ⇒ E1BB **必达**（后继）。
- **净效应裁定再确认**：E0FA 瞬态置 bit4=1、清 bit3 → E1BF `&=$80` 覆盖 ⇒ 终态 `$26 = 进入前 $26 & $80`。
  **bit4 终值=0**、bit3=0、bit7 原样保留。**AR69-1 裁定 `$26&=$80` 成立**（A 级复现）。

### 2.5 核验 5：grep 全仓残留"bit4 置位"旧模型 —— 通过（硬断言清净）

grep 全仓（`bit4 置位|bit4=1|== 0x10|置位.*0x10|ORA #$10` 等）逐条甄别：

- **断言层**：`f12` 已改 `& 0x10 == 0`（§2.2）；无任何 test 断言 `$26` revive_flags bit4==0x10。`death_chain.rs:211 st.hit_flags=0x10`、`flow.rs:402 hit_flags|=0x10` 系 **`$0668` 命中标志 bit4**（另一字段，`D64C ORA #$10`），非 `$26`，不属旧模型。
- 其余 `0x10` 命中（`FLAG_B`、`BTN_START`、`MENU_BTN_START`、`terrain loop_cnt`）均与 `$26` 无关。
- ⇒ **"bit4 置位"旧模型硬断言清净，通过**。
- 软残留（注释/摘要）flow.rs:485/488 + spec:141/158/176 属旧"缺口/单写"模型（非"bit4 置位"硬断言），同根归入 **AR70-1** 一并清理。

### 2.6 核验 6：AD69-2 挂注（R-G4 并入）在文 —— 通过

`docs/findings/adversarial-review-69.md`（随 `7d7a964` 归档）内 AD69-2 挂注带 R-G4 并入标注在文：

- L127："挂起新探针（AD69-2）：`bank00:9C5D $018F:=4`（bit4 门控块）是否经续关路径间接可达…**（并入 R-G4 场景组）**"
- L134："AD69-2（低，挂起）…指派 oracle 场景组：continue 动态 trace 查 `$018F`（**R-G4 场景一并采** GAME OVER 屏）"

**可选提示（不阻断）**：AD69-2 目前仅存于复核归档文；A2 findings R-G4 行（`game-over-continue.md:456`）与 spec §2 遗留假设尚未回挂 `$018F:=4` 探针，
后续 oracle R-G4 场景组读台账时看不到此探针。建议顺手把 AD69-2/$018F 探针并入 A2 R-G4 行（一句话），避免探针遗失。

## 3. findings 四分类与修正建议

| # | 现象 | 分类 | 处理 |
|---|---|---|---|
| **AR70-1（中低，必修）** | commit 自述"A2 §3.2 原地 dated 更正"**未落**：A2 `game-over-continue.md` 未被改、全文无 E1BB、§3.2/§2 `$26` 行无 dated 更正；且旧"缺口/未接 E1BB"模型残留 spec:141/158/176 + flow.rs:485/488，与净效应 `&=0x80` 矛盾 | 文档收口缺口（证伪更正文化 §3.6 未完成 + commit overclaim）；零行为影响 | 回 spec-writer/porter：① A2 `game-over-continue.md` §2 `$26` 行（L153）+ §3.2 表原地加 `【2026-10-14 更正（复核 #69）】`（E0F4 瞬态→E1BB 终态两写点，净 `$26&=$80`，bit4=0）；② spec:141/158/176 与 flow.rs:485/488 的"未接 E1BB / E1BB 缺口 / 规格缺口"改为"已裁定写点（净 `$26&=$80`）"或 dated 历史注（删残缺断句）；③ 复跑 `cargo test`+l2+gates 复绿 |
| AD70-2（低，可选） | AD69-2/$018F 探针未回挂 A2 R-G4 行/ spec 遗留假设，仅存复核归档 | 探针收口（挂起项） | 一句话并入 A2 `game-over-continue.md:456` R-G4 行 |

**修正建议（最小修复批，纯文档、零行为风险）**：
1. A2 `docs/findings/game-over-continue.md`：
   - §2 `$26 bit3` 行（L153）补 dated 更正：`【2026-10-14 更正（复核 #69）】E0F4-E0F8 为瞬态写，E1BB-E1BF `$26&=$80` 为终态写 ⇒ 净效应 bit4=0`。
   - §3.2 复位范围表补 `$26` 处置行（或在既有行尾）同型标注。
2. 清残留（同根"未接/缺口"旧模型）：`docs/spec/game-over-continue.md:141/158/176`、`crates/game-core/src/flow.rs:485/488` —— 将"未接 E1BB / E1BB 缺口 / 规格缺口"改"已裁定（净 `$26&=$80`）"；flow.rs:488 残缺断句"（历史）／spec-writer）"删整或补全为 dated 历史注。
3. 复验：`cargo test --workspace`(135) + `tools/diff_subroutines.py`(CN/JP 各 121) + `gates/cli.py check`(12/12)。断言不变 ⇒ 测试数不动、ratchet 不动。

## 4. 对拍独立复跑（三件 + 双版贴数）

| 项 | 命令 | 独立实测输出 |
|---|---|---|
| workspace 测试 | `cargo test --workspace` | **TOTAL passed: 135**（death_chain 10 + flow 7 + 其余 crate 求和；`test result: ok. 0 failed` 全绿） |
| l2-diff CN | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过**（golden==ROM 执行==game-core 同名函数，三者缺一不可） |
| l2-diff JP | `python3 tools/diff_subroutines.py --rom reference/jp/gunnac_jp.nes` | **L2-DIFF: 121/121 通过** |
| 门禁 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（含 0 静默跳过；`rust-tests 135（基线 135）`、`ratchet:rust_test_count 135/135 OK`） |

对拍三件 + 双版 + gates 全绿 ⇒ AR69-1 行为修复无回退、测试面无缩减、基线 ratchet 无降。

## 5. 附录：复现命令与留痕

```bash
# 对拍三件（本复核实跑）
cargo test --workspace                                    # TOTAL 135
python3 tools/diff_subroutines.py                          # 121/121
python3 tools/diff_subroutines.py --rom reference/jp/gunnac_jp.nes   # 121/121
python3 gates/cli.py check                                 # PASS=12 FAIL=0 SKIP=0 TOLERATED=0

# 核验 4 两写点字节 + 执行序独立直读
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
rd=lambda cpu,n: prg[0x1E000+(cpu-0xE000):0x1E000+(cpu-0xE000)+n]
print("E0F4-E0FA:", rd(0xE0F4,7).hex(' '))   # a5 26 29 f7 09 10 85
print("E1BB-E1BF:", rd(0xE1BB,5).hex(' '))   # a5 26 29 80 85
seg=rd(0xE1B6,0x1C); print("E1B6..E1D1 控制流:", [hex(b) for b in seg if b in (0x4C,0x6C,0xD0,0xF0,0x10,0x30,0x90,0xB0)] or "NONE(线性必达)")
PY

# 核验 3b A2 未落（git 双证）
git show 7d7a964 --name-only | grep findings/game-over-continue || echo "A2 未被触及"
git log --oneline -5 -- docs/findings/game-over-continue.md   # 末次 8bee3c6（修复批之前）
grep -c E1BB docs/findings/game-over-continue.md              # 0
```

- 本复核未改动任何被审文件；唯一产出=本文件。
- mutation/字节留痕：ROM 只读；两写点字节 + 线性执行序为可复现 A 级证据（命令见上）。

---
**Verdict: needs_revision**
- AR69-1 **行为实质全落地**：两写点净效应 `$26&=$80`（bit4=0）经 ROM 直读再确认，代码/测试/spec 三处一致，对拍三件+双版+gates 全绿。
- **唯 1 项必修 AR70-1**（纯文档收口，零行为影响）：A2 findings §3.2/§2 `$26` 行 dated 更正未落（commit 自述 overclaim），且残留"未接 E1BB / 缺口"旧模型 spec:141/158/176 + flow.rs:485/488 与实现矛盾。
- 六项：1✓ / 2✓ / 3✗(3b) / 4✓ / 5✓ / 6✓ —— 除 3b 外全落地；3b 补齐后可复判 pass。
