# 对抗复核：l5-0160-batch-semantics（2026-10-20，只读复核）

VERDICT: **pass**（附 2 条 nit）——3 项检查 2 过 1 记录在案（git 树不干净，但非本文件所致）。

1) dated 撤回：**在文内 ✓**。§8（L185-209）保留初版原文并以 `【2026-10-20 更正】` 原地撤回 4 条主张（L187-205：「写脉冲帧/v=3 写点」「v=1@BBBD=写点族」「EED4 逐帧请求」「帧末单快照正确」）；头部 L3-6 亦有 dated 更正，历史主张未删。nit：L3「2026-01-32」为不存在的日期，疑为「2026-10-20 01:32」之误（对照 L185）。
2) pc 归因 vs seed：**对得上 ✓**。`EED1`→seed L89184 `bank15:EED1 AD 60 01 LDA $160`（`EED4`=后取指 PC，L89185 `D0 FB BNE`）；`F0DC`→L89442 `AD 60 01 LDA $160`（`F0DF`=后取指 PC）；`88A0`→L1142 `bank00:88A0 0D 60 01 ORA $160`；`BBBA`→L23309-23317 字节流 `AD 60 01 09 01 8D 60 01 60` 与文内引文逐字节一致（BBBD=LDA 后取指、BBBF=STA，自洽）。nit：seed 将 BBBA-BBC2 列为数据字节行（`$AD`…），文内「bank3:BBBA … LDA $160」系解码归因而非 seed 原行（文内自称"seed 字节流"，可接受，宜在 spec 引用时注明）。旁证抽查全中：EEDF/EEE1（L89191-92）、EECE/EED9/E8E0/E03E/EFA2/F09A/F14B/F1F9/F0E1/F0F7。
3) `git status --porcelain`：**不干净 ✗**——`M  docs/findings/team-ops-ledger.md`（已暂存改动，复核前即存在，与 l5-0160 无关）；本复核文件为新增 untracked。按指令未执行任何 git 变更操作。
