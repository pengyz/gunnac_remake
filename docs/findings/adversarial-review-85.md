# 对抗复核 #85 — T1 工作全集盘点（rewrite-workset，commit bd9ee8c）

> 日期：2026-10-15｜角色：adversarial-reviewer（证伪立场；实施方=decomp-analyst≠审核方）
> 被审：commit `bd9ee8c`（docs/spec/rewrite-workset.md + data/status/rewrite-workset.json +
> tools/workset_inventory.py）。方法：全部关键数字自写脚本从 seed/ROM/镜像 JSON **独立重导**，
> 不调 workset_inventory.py。拆段落盘（每段一 commit）。编号注：#84 已被批次 1+2 复核占用，T1 顺延 #85。

## 段 A：三方对账差集独立重导（可查点①）——**全等** ✅

### A.1 方法（独立性）
自写入口枚举器：解析 `reference/seed/gunnac.asm` 全部 91,558 行 bankNN:AAAA 指令/数据行
（字节场→助记符→操作数），按 E1-E5 规则并集；far-call 与向量**从 ROM 原字节直读**；
对照面 = `data/status/rewrite-workset.json` 逐 bank 逐地址 diff + `analysis/ida/bankNN.json` xrefs。

### A.2 逐项裁定

| 主张 | 独立重导 | 裁定 |
|---|---|---|
| **seed 入口全集 = 394 = 394 段** | ±n 归一后并集 = **394**，与镜像 JSON **逐 bank 逐地址全等**（16/16 bank 零差异） | ✅ |
| **逐 bank 分布** 65/9/7/12/1/3/3/16/1/1/7/41/1/3/121/103 | 逐 bank 集合全等 ⇒ 分布全等 | ✅ |
| E1 窗基址 16 | bank_win 枚举 = 16（$8000×8/$A000×5/$C000/$E000） | ✅ |
| E2 同窗 279 + 固定窗 135 | 同窗唯一 279（含 byte_ 标签形）、固定窗唯一 **135 精确** | ✅ |
| E3 向量 NMI=$EF85/RESET=$E000/IRQ=$F20B | **ROM file 0x2000A 直读** = EF85/E000/F20B | ✅（A 级字节） |
| E4 far-call 目标 12（站点 18） | **ROM 全扫 `20 49 E6` = 18 站点**，内联 (R6,R7,lo,hi) 解出 **12 唯一目标**（bank11×4/bank02×3/bank00×5） | ✅（与 rom-forensics §2 一致） |
| E5 bank11:$8000 JMP 表 7 项 | $88A3/$88A7/$890B/$8016/$8894/$889B/$8E2C = 7 | ✅ |
| **only_ida = 0**（seed ⊇ IDA 294） | 逐 bank IDA xref 集 − 我的集合 = **空** | ✅ |
| **增补 100** = far-call 10 + 跨 bank 固定窗 90 | 我的 only_seed 合计 = **100 精确**（构成比例与声称一致：far-call 目标 12 中 2 个 IDA 已见 ⇒ 增补 10 ✓） | ✅ |
| remap 276→192 唯一体（f_005..f_018 同 sub_8185） | 抽验：remap symbols 的 cpu 字段=所属函数体起点（别名塌缩口径）与 JSON 抽样一致 | ✅（抽验） |

### A.3 差异溯源记录（两处均裁为我方口径问题，非被审错）
1. **±n 位移项**：`loc_XXXX±n` 操作数我首跑未施位移 ⇒ 6 个 bank 各差 1-3 项（如 `loc_845A-2` 我记 8458）。
   **JSON 归一化（标签±位移=真目标）为正确口径**——修正后全等。顺带裁 **bank15 0xEC11 疑云**：
   `JSR loc_EC11+1` 实指 **EC12**，"我多 EC11"即此伪差，被审记录无误。
2. **同窗 278→279**：`byte_BEBD` 标签形目标（JMP byte_BEBD）补入后 279 ✓。

### A.4 本段结论
① 三方对账差集**全部数字精确成立**（394/279/135/100/only_ida=0/18 站点/12 目标/3 向量）。
段 A 不单独判 verdict——段 C 统一给出。

---

## 段 B：L2 归段校验（②）+ 分类指纹抽 10（③）

### B.1 ② L2 归段校验（自算二分，不调 workset_inventory）——**全数精确** ✅

| 主张 | 自算 | 裁定 |
|---|---|---|
| 121 例 / 29 子程序 / 15 段 | **121 / 29 / 15**（逐例 entry→段区间二分） | ✅ |
| unmapped L2 entries = 0 | 自算 unmapped=0；JSON `l2_unmapped_entries=[]` | ✅ |
| 3/29 entry=段首 | **恰 3**：`pickup_handler bank2:922B`、`money_add_event bank14:DBCE`、`bcd_sub_ammo bank15:ED03`——与声称**逐名一致** | ✅ |
| 其余 26 名=段内手选片段 | 29−3=**26** | ✅ |
| 15 段合计 6,865B | 自算 **6,865** 精确 | ✅ |

口径核：§3.2 "段级状态只证明该段有对拍面，片段语义另见 README"——**口径诚实**（26 名片段不冒充段级语义）✓。

### B.2 ③ 分类指纹抽 10（自算 fp×R 规则 vs 镜像）

抽样含 4 个校准锚（ui_scroll_write/EF85 NMI/bank11 音频簇/pickup_handler）+6 段随机（跨 5 类）：

| 段 | 声称 class(rule) | 指纹对比（8 字段） | 自算 class ⇒ 判类 |
|---|---|---|---|
| bank01:A92E-AB6F | 图形(R-G) | ≠ram_writes（写形态口径差 1） | R-G ✓ 一致 |
| bank15:EF85-F02F | 图形(R-G) | ≠mapper_write（13 vs 16） | R-G ✓ 一致 |
| bank11:8166-81B3 | 音频(R-A) | ≠apu_write（3 vs 2） | R-A ✓ 一致 |
| bank02:922B-92C4 | 待定(R-X) | **✓ 全等**（0 指令/154 数据行） | R-X ✓ 一致 |
| bank00:8000-8034 | 引擎(R-E) | **✓ 全等** | R-E ✓ 一致 |
| bank14:DBCE-DDCE | 引擎(R-E) | ≠insn/data/ram_writes | R-E ✓ 一致 |
| bank15:ED03-ED26 | 待定(R-?) | **✓ 全等** | R-? ✓ 一致 |
| bank03:A039-A0FB | 引擎(R-E) | **✓ 全等** | R-E ✓ 一致 |
| bank07:A000-A0A5 | 引擎(R-E) | **✓ 全等** | R-E ✓ 一致 |
| bank13:8000-82E4 | 引擎(R-E) | ≠insn/data/mapper/ram_writes | R-E ✓ 一致 |

**裁定**：指纹字段级 5/10 全等、5/10 残差（我方写形态口径窄：间接/索引写与部分 store 形态覆盖不同），
但 **10/10 分类判定全部一致**（R 规则首条命中点未被残差改变）——③ 成立。差异溯源两笔均记我方口径：
① 我方助记符集漏 INY/INX/DEX/DEY（bank01 的 23 行=20+1+2 精确对账，**被审 insn 计数正确**）；
② 写形态覆盖（间接 `(zp),Y` 写、mapper 写经索引形）我方计数窄——**不影响任何 R 规则命中**。
分类计数算术自洽：311+2+17+6+58=**394** ✓（段 C 字节面复算另核）。
