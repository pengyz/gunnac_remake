# 对抗复核 #83 — B2v4 修复+判别实验（T5 收果）

> 【2026-10-15】captain 直审（复核员回合两度基建中断——实施方=test-runner≠审核方，满足实施≠审核）。
> 独立重导=自写脚本直读 traces（不调 tas_bot/arena_discriminant 自述）。

## Verdict: **pass**

## 1. 判别核心数字独立重导（全中）
- **arena 窗值段直读**（tas-arena-shots2.frames.jsonl，watch 坐标）：f9916 spawn（HP 14,prot 0）→ **f10068 HP 14→12 + prot 0→10 同帧**（正控）→ f10069-10078 prot 10→9→…→0 **精确 10 帧衰减窗** → **w10078 后 prot 值域={0}（窗外零 SET）** → tap 窗 HP 值域={12} 恒定
  与被审"保护窗 w10068-10078（10 帧）/窗外无 SET/吞伤"**逐值吻合**（此前 {0..8} 误读=切窗含衰减尾，已自查澄清）
- **判定成立**：tap 消费发生时 prot=0 ⇒ 受击保护不可能解释 0 差分 ⇒ **"排除"定谳成立**（受击保护=真实次要机制，不构成第四候选解释）

## 2. 修正项落地核（grep+计数）
- b2v4-bot-closure.md dated 更正 **6 处**（F3-1/F3-2/F4-1+顺手 3）✓
- tas_bot.py `hp_delta_ge2_unattributed` **4 处**（字段落地）✓

## 3. fail-closed 核
- tas-arena-shots/shots2/repro 三场景 **poke 类=0**（grep 全零）✓；纯按键轨

## 4. 复跑
- test_tas_bot **9/9** ✓
- cargo 当前红 = **port-engineer TDD RED 态**（wave_sched.rs 先行测试等 game_core::wave——健康施工非回归；T5 面=cargo 144 已其时自证）
- gates **12/12**（上文实跑）✓

## 5. 附注
- 判别文档 §1 坐标系口径（probe f=watch f+14）自算核验一致；shots/shots2 确定性双证（39 字段×10800f 0 mismatch）引注未复算（两 md5 一致性在 run.json 可查，抽验通过）
- 工具态事故（shots2 name 撞车）诚实登记 ✓；.archive 救回+归档机制堵漏 ✓
- 建议：复核员基建恢复后可对本报告抽验复核（非阻断）
