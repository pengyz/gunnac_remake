#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/s5_anchor_rederive.py —— G-S5-6 真帧锚重导（TRACE-DIFF 入库复现命令，复核 #74 F3）。

从 `analysis/traces/*.frames.jsonl` 原始 trace 逐判定式机器重导 docs/findings/gs5-6-dynamic-replay.md
§3 的 8 个对拍锚（RAM 5 + 截图/capture 3），并复算两处复核 #74 更正口径（F1/F2）：

  ① TRACE-DIFF（§3 对拍口径）：5 RAM 判定式锚 + 3 截图锚 ⇒ `TRACE-DIFF: ALL MATCH (8/8)`
  ② F1 更正口径（gate-26-bit4 判定式 = 关末语义三联）：Run A 自然触发未观测 + bit4 过渡窗定界
  ③ F2 更正口径（确定性对账）：c1↔c2 f1..3200 段 0 差异（f3201..5000 设计性分歧）/
     c2↔anchor 全 5000f 0 差异 / gameplay-stage1↔full 7 字段×2400f 0 差异

只读重导，不改任何 oracle/门禁/数据。trace 缺失时提示用
`python3 tools/scenario_run.py run <场景>` 重跑（analysis/ 为可再生工作区）。

用法：
    python3 tools/s5_anchor_rederive.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRACES = ROOT / "analysis" / "traces"

RESULTS: list[tuple[str, bool, str]] = []


def check(name: str, ok: bool, detail: str) -> None:
    RESULTS.append((name, ok, detail))
    print(f"[{'ok' if ok else 'FAIL'}] {name}: {detail}")


def load_frames(run: str) -> dict[int, dict]:
    p = TRACES / f"{run}.frames.jsonl"
    if not p.exists():
        sys.exit(f"[缺失] {p} —— 重跑：python3 tools/scenario_run.py run {run}")
    out: dict[int, dict] = {}
    with p.open() as f:
        for line in f:
            line = line.strip()
            if line:
                d = json.loads(line)
                out[d["f"]] = d["w"]
    return out


def load_captures(run: str) -> list[dict]:
    p = TRACES / f"{run}.captures.jsonl"
    if not p.exists():
        sys.exit(f"[缺失] {p} —— 重跑：python3 tools/scenario_run.py run {run}")
    return [json.loads(x) for x in p.read_text().splitlines() if x.strip()]


# ── ① TRACE-DIFF：8 对拍锚逐判定式重导 ──────────────────────────────────────


def rederive_anchors() -> None:
    an = load_frames("gs5-stg01-clear-anchor")
    caps = load_captures("gs5-stg01-clear-anchor")

    # RAM 锚 5/5（判定式 = findings §3 表 / anchors.jsonl predicate）
    w = an[2401]
    check(
        "RAM stg01-clear f2401",
        bool(w["flags27"] & 0x80) and w["countdown2f"] == 50 and an[2400]["flags27"] == 0,
        f"flags27={w['flags27']:#04x} ∧ countdown2f={w['countdown2f']}（f2400 flags27=0）",
    )
    w = an[2794]
    check(
        "RAM flyout-vector f2794",
        w["countdown2f"] == 0 and w["vec41c"] == 0xA4 and w["vec438"] == 0x0D and an[2793]["countdown2f"] == 1,
        f"countdown2f={w['countdown2f']} ∧ vec41c={w['vec41c']} ∧ vec438={w['vec438']}（f2793=1，判定帧唯一）",
    )
    w = an[3066]
    check(
        "RAM gate300-open f3066",
        w["gate300"] == 0x10 and not (w["flags26"] & 0x10),
        f"gate300={w['gate300']:#04x} ∧ flags26={w['flags26']:#04x}（bit4 清）",
    )
    check(
        "RAM inc-180 f3096",
        an[3096]["stage_no"] == 2 and an[3095]["stage_no"] == 1,
        f"stage_no {an[3095]['stage_no']}→{an[3096]['stage_no']}（INC $180 写帧）",
    )
    w = an[3116]
    check(
        "RAM stg02-shop f3116",
        w["stage_no"] == 2 and w["player_state"] == 0,
        f"stage_no={w['stage_no']} ∧ player_state={w['player_state']}（$9BC3 init）",
    )

    # 截图锚 3/3（capture 记录 + 截图文件实存）
    for f, tag, name in [
        (3400, "exit-fade4", "stg02-exit"),
        (3450, "stg02-start", "stg02-start"),
        (3900, "stg02-combat", "stg02-combat"),
    ]:
        rec = [c for c in caps if c.get("f") == f and c.get("tag") == tag]
        shot_ok = all((ROOT / str(c.get("shot", ""))).exists() for c in rec) if rec else False
        ram_ok = all((ROOT / str(c.get("ram", ""))).exists() for c in rec) if rec else False
        check(
            f"SHOT {name} f{f}",
            bool(rec) and shot_ok and ram_ok,
            f"capture tag={tag} 记录 {len(rec)} 条 + 截图/RAM dump 实存",
        )

    n_ok = sum(1 for _, ok, _ in RESULTS if ok)
    print(f"\nTRACE-DIFF: {'ALL MATCH' if n_ok == len(RESULTS) else 'MISMATCH'} ({n_ok}/{len(RESULTS)})")


# ── ② F1 更正口径：gate-26-bit4 判定式 = 关末语义三联（复核 #74）──────────────


def check_f1() -> None:
    """关末语义三联 = flags27 bit7 入场（E309）+ $2F 50→0 倒数环（8f cadence DEC）+ INC $180（E37C）。
    Run A（gs5-stg01-full 15000f）三联未出现 ⇒ 关末语义自然触发未观测；
    $26 bit4 自然置位窗 = f913..1192（菜单→开 stage 过渡窗，非关末语义）。"""
    A = load_frames("gs5-stg01-full")
    b4 = sorted(f for f, w in A.items() if w["flags26"] & 0x10)
    runs, cur = [], [b4[0]] if b4 else []
    for x in b4[1:]:
        if x == cur[-1] + 1:
            cur.append(x)
        else:
            runs.append((cur[0], cur[-1]))
            cur = [x]
    if cur:
        runs.append((cur[0], cur[-1]))
    check(
        "F1 bit4 过渡窗定界（Run A）",
        runs == [(913, 1192)],
        f"flags26 bit4 置位窗 = {runs}（f913..1192=菜单→开 stage 过渡窗；非「全程 0」）",
    )
    w1192, w1193 = A[1192], A[1193]
    check(
        "F1 过渡窗语义（开 stage）",
        (w1192["flags26"] & 0x10) and not (w1193["flags26"] & 0x10) and w1193["intro_ctr"] == 64,
        f"f1193 bit4 清 + intro_ctr={w1193['intro_ctr']}（关始引入；窗口期间 countdown2f={w1192['countdown2f']} 非倒数环）",
    )
    ca_nz = sum(1 for w in A.values() if w["trig_ca"] != 0)
    check("F1 $CA 全程 0（此条原断言成立）", ca_nz == 0, f"trig_ca 非零帧数 = {ca_nz}")
    # 三联腿②：8f cadence 倒数环（50→0）
    def has_ring(w: dict[int, dict]) -> int | None:
        for f in sorted(w):
            if w[f]["countdown2f"] == 50 and all(
                f + 8 * k in w and w[f + 8 * k]["countdown2f"] == 50 - k for k in range(1, 50)
            ):
                return f
        return None
    # 三联腿③：INC $180
    stage_tr = [f for f in sorted(A) if f > 1 and A[f]["stage_no"] != A[f - 1]["stage_no"]]
    ring = has_ring(A)
    check(
        "F1 关末语义三联未出现（Run A）",
        ring is None and not stage_tr,
        f"倒数环（50→0·8f DEC）={'有@f'+str(ring) if ring else '无'}；INC $180 转移={stage_tr or '无'}"
        "（flags27 腿：轻量面未 watch $27，经同前缀 anchor run f913-1192 同窗 flags27=0 旁证）",
    )


# ── ③ F2 更正口径：确定性对账（复核 #74）──────────────────────────────────


def check_f2() -> None:
    c1, c2 = load_frames("gs5-stg01-clear-c1"), load_frames("gs5-stg01-clear-c2")
    an = load_frames("gs5-stg01-clear-anchor")

    def mism(a: dict, b: dict) -> list[int]:
        keys = sorted(set(next(iter(a.values()))) & set(next(iter(b.values()))))
        out = []
        for f in sorted(set(a) & set(b)):
            if any(a[f].get(k) != b[f].get(k) for k in keys):
                out.append(f)
        return out

    bad12 = mism(c1, c2)
    pre = [f for f in bad12 if f <= 3200]
    post = [f for f in bad12 if f >= 3201]
    check(
        "F2 c1↔c2 共享段（18 字段 × f1..3200）",
        not pre and post == list(range(3201, 5001)),
        f"f1..3200 mismatch={len(pre)}；f3201..5000 mismatch={len(post)} 帧（设计性分歧：c1 B 试探 vs c2 down 导航）",
    )
    bad2a = mism(c2, an)
    check("F2 c2↔anchor 全程（18 字段 × 5000f）", not bad2a, f"mismatch={len(bad2a)}")

    # gameplay-stage1 ↔ full：7 字段 × 2400f（watch 映射取值；wram blob=$6000-7FFF 非 $0000 基址）
    gp, fu = load_frames("gameplay-stage1"), load_frames("gs5-stg01-full")

    def gp_get(w: dict, addr: int) -> int:
        if addr == 0x401:
            return w["game_action_mode"]
        if addr == 0x188:
            return w["intro_ctr"]
        if addr == 0x18E:
            return w["money_disp_copy"]
        if addr == 0x18:
            return bytes.fromhex(w["zp"])[0x18]
        if addr == 0x26:
            return bytes.fromhex(w["zp"])[0x26]
        if addr == 0x300:
            return bytes.fromhex(w["iram"])[0x000]
        if addr == 0x400:
            return bytes.fromhex(w["iram"])[0x100]
        raise KeyError(addr)

    def fu_get(w: dict, addr: int) -> int:
        return {
            0x401: w["slot1_state"], 0x188: w["intro_ctr"], 0x18E: w["money_disp_copy"],
            0x18: w["controller_held"], 0x26: w["flags26"], 0x300: w["gate300"],
            0x400: w["player_state"],
        }[addr]

    fields = [0x401, 0x188, 0x18E, 0x18, 0x26, 0x300, 0x400]
    bad = sum(
        1
        for f in range(1, 2401)
        for a in fields
        if gp_get(gp[f], a) != fu_get(fu[f], a)
    )
    check(
        "F2 gameplay-stage1↔full（7 字段 × 2400f）",
        bad == 0,
        f"mismatch={bad}（zp/iram blob 按场景 watch 映射解：wram=$6000-7FFF SRAM 区非 $0000 基址）",
    )


def main() -> int:
    rederive_anchors()
    print("\n── 复核 #74 更正口径复算（F1/F2）──")
    n0 = len(RESULTS)
    check_f1()
    check_f2()
    ok = all(x[1] for x in RESULTS)
    print(f"\nSUMMARY: {sum(1 for _, o, _ in RESULTS if o)}/{len(RESULTS)} 通过"
          f"（对拍 {n0} + 更正口径 {len(RESULTS) - n0}）→ {'ALL MATCH' if ok else 'MISMATCH'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
