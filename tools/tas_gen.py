#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/tas_gen.py —— B2 TAS 输入生成器（规格：docs/spec/tas-input-dsl.md）。

输入 = 波次时间线（B1：`tools/wave_timeline.py` 调度流/出敌事件）+ 已证参数
（移动常量/射击门控/炸弹消耗/命中包络）→ 输出 = **纯输入轨**（scenario DSL 的 press/hold）。

硬性红线（AGENTS.md §3 + README 写注入纪律）：
- 本工具**没有任何写内存/改状态能力**：输出面只允许 `press`/`hold` 两个动词；遇到 plan 里的
  poke 类指令直接拒绝（fail-closed）；**未实现指令同样显式拒绝，不静默吞**
  （when 子句=未实现档位，见下；复核 #78 F1）。
- 规则只来自已证参数（R-tas-1..7，见规格 §2）；弹速表语义（Wt-6）/敌弹 RAM 布局/boss 血量
  等未证语义**不参与决策**，生成报告显式列出。

用法：
  python3 tools/tas_gen.py gen <plan.tas> [-o oracle/scenarios/]
  python3 tools/tas_gen.py timeline [--trace <frames.jsonl>]
  python3 tools/tas_gen.py record-to-scenario <inputs.jsonl> <name> [-o …]
  python3 tools/tas_gen.py report <plan.tas>

plan DSL（docs/spec/tas-input-dsl.md §1.2）：
  stage <N> / frames <N> / prefix <场景名> / fire <起>-<止> / bomb budget <N>
  bomb at <帧,帧,…> / band x <lo>-<hi> y <lo>-<hi> / sweep period <F> [phase <F>]
  when <pred> then <action>       # 【未实施·显式拒绝】生成期展开档位未建（spec §1.2 更正注）；
                                  #   子句动作动词白名单={hold,press,avoid}，白名单外（含 poke 类）→ PlanError
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import wave_timeline as wv  # noqa: E402  调度流字节语义唯一实现

SCENARIO_DIR = ROOT / "oracle" / "scenarios"

# ---- 已证常量（规格 §2；出处逐条注释，勿改数值）----
CLAMP_X = (0x0E, 0xF3)      # R-tas-3：bank03:A0D0-A0E3（C-clamp-x）
CLAMP_Y = (0x14, 0xDF)      # R-tas-3：bank03:A09D-A0B0（C-clamp-y）
VEL_PX_PER_FRAME = 2.8125   # R-tas-3：速度对 (208,2)/(48,253)（C-vel-h）
HIT_ENVELOPE = 12           # R-tas-5：命中包络候选 |dx|≤12,|dy|≤12（hit-sampling-h16 §2，B 级）
AVOID_MARGIN = HIT_ENVELOPE + 4  # 包络 +4px 余量（经验值，报告显式标注）
BOMB_AMMO_MAX = 20          # R-tas-2：商店「导弹20个 上限」+ $018F
BOMB_CONSUME = (1, 2, 3, 2)  # R-tas-2：bank03:$A336（C-bomb-consume）
# R-tas-2：投放去重窗下界（生成期排程去重启发式，**不是寿命常量**）。
#   出处：chain-A 实测在途窗 209f；他链 254f → 按 209 去重（保守下界）。
#   ⚠ 与 player-movement A5「209f 寿命=行为链合成值，移植端禁止硬编码 209」不冲突：
#   本常量只在生成期排程去重用，不消费 $0401 在途寿命语义，移植端不得当寿命引用
#   （A1 改名避嫌：复核 #78 A1——原名 BOMB_DEPLOY_FRAMES 易误读为"投放=209f 寿命"）。
BOMB_DEPLOY_DEDUP_HINT = 209
FIRE_REPEAT_FRAMES = 7      # R-tas-1：$39=8 重复计时器（fire-hold 实测每 7 帧一发）

# ---- tick→帧映射锚（实测：analysis/traces/b2-fire-probe.frames.jsonl，$84/$85 逐帧）----
# Wt-2 已裁定（2026-10-15，复核 #78 §4）：**tick = 16px 滚动进度**（A 级——三段全等
#   8f×2px/f = 2f×8px/f = 16f×1px/f = 16px；$8B 逐帧 −2 直证；bank00:8D3A 减速表逐指令吻合），
#   tick 频率随滚动速度变化（8f/2f/ramp/16f 四段），非恒定 8f。
#   锚值 = 脚本层 tick 计数器 $84；调度流累计 tick = $84 + SCHED_TICK_OFFSET（实测 +1，
#   B1 §2.2 锚点校验：sched +33t↔f1341 时 $84=32、+161t↔f2365 时 $84=160 → 7 锚全等）。
TICK_MAP_ANCHORS = [
    (16, 1193), (17, 1221), (192, 2621), (288, 2813),
    (350, 2957), (400, 3143), (450, 3833), (500, 4633), (558, 5561),
]
SCHED_TICK_OFFSET = 1
# 超出实测段按 16f/tick 线性外推（假设 W-tas-3，报告显式）。
#   补注（复核 #78 §4）：tick558-815 实测 257 tick 全 16f，且 tick=16px 滚动进度已证（Wt-2 关闭）——
#   16f/tick 在 1px/f 恒速段为实测值；外推仅覆盖 W-tas-3 尾段 tick 816→1232（未实测，arena 停滚期
#   tick 不推进、rec12 set-speed 后区间未测）→ 该段仍是假设。
TICK_EXTRAP_F_PER_TICK = 16.0


class PlanError(Exception):
    pass


# when 子句动作动词白名单（生成期展开档位的合法动作面；当前档位=显式拒绝，见 parse_plan）。
#   白名单外动词（含 poke 类写注入）→ PlanError；白名单内动词 → when 未实现拒绝（不展开）。
WHEN_ACTION_WHITELIST = ("hold", "press", "avoid")
POKE_CLASS_VERBS = ("poke", "poke_every", "snapshot", "restore")


def safe_name_component(name: str, what: str = "name") -> str:
    """A1（复核 #78）：单段路径消毒——拒绝路径分隔符/点段/绝对路径/空名，防越出 SCENARIO_DIR。

    `--name`/`prefix`/record-to-scenario 名字只允许单个路径段（如 `tas-stg01-v2`）。
    """
    if not isinstance(name, str) or not name or name in (".", ".."):
        raise PlanError(f"{what} 非法（空名/点段）: {name!r}")
    if "/" in name or "\\" in name or "\x00" in name:
        raise PlanError(f"{what} 含路径分隔符/非法字符（只允许单段名）: {name!r}")
    if name.startswith("."):
        raise PlanError(f"{what} 不能以点开头: {name!r}")
    if Path(name).name != name:
        raise PlanError(f"{what} 不是单一路径段: {name!r}")
    return name


# ---------------------------------------------------------------- 时间线

def sched_timeline(stage: int, maxrec: int = 200):
    """B1 §2.2 调度流记录 → {tick, spawns:[(type, members, interval, param), …], raw}。"""
    d = wv.load()
    out = []
    for r in wv.iter_sched_records(d, stage, maxrec=maxrec):
        if r["truncated"] or r["loop"]:
            out.append({"tick": r["tick"], "spawns": [], "raw": r})
            break
        spawns = []
        for it in r["items"]:
            if it[0] == "ctrl" and it[1] in (1, 2, 11, 12):
                # ctrl1/11: 定型 $2E/$2C + 4 参；ctrl2/12: 定型 $2F/$2D + ($3E) 参 + 4 参
                # 四参语义（B1 §2.2，A 级三记录签名）：(组员type, 组员数, 出敌间隔f, 参数)
                a = it[3]
                spawns.append({"head": it[2], "member_type": a[0], "count": a[1],
                               "interval_f": a[2], "param": a[3]})
            elif it[0] == "set" and it[1] == 10:
                spawns.append({"head": it[2], "member_type": it[2], "count": 1,
                               "interval_f": 0, "param": it[3]})
        out.append({"tick": r["tick"], "spawns": spawns, "raw": r})
    return out


def tick_to_frame(tick: int, anchors=None, extrap=None, source: str = "sched") -> float:
    """source='sched'：调度流累计 delay tick（内部先换算到 $84 口径）；'script'：$84 本身。"""
    if source == "sched":
        tick = tick - SCHED_TICK_OFFSET
    """tick（调度流累计 delay tick 口径）→ 帧。实测段线性插值，尾段线性外推。"""
    anchors = anchors or TICK_MAP_ANCHORS
    extrap = TICK_EXTRAP_F_PER_TICK if extrap is None else extrap
    if tick <= anchors[0][0]:
        return anchors[0][1] + (tick - anchors[0][0]) * 8.0
    for (t0, f0), (t1, f1) in zip(anchors, anchors[1:]):
        if t0 <= tick <= t1:
            k = (f1 - f0) / (t1 - t0)
            return f0 + (tick - t0) * k
    t0, f0 = anchors[-1]
    return f0 + (tick - t0) * extrap


# ---------------------------------------------------------------- plan DSL

def parse_plan(path: Path) -> dict:
    plan = {"stage": None, "frames": None, "prefix": None, "fire": None,
            "bomb_budget": BOMB_AMMO_MAX, "bomb_at": [], "band": None,
            "sweep_period": 120, "sweep_phase": 0, "when": [], "capture_every": None,
            "source": str(path)}
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        toks = line.split()
        op, args = toks[0], toks[1:]

        def err(msg):
            raise PlanError(f"{path}:{lineno}: {msg}（行: {raw.strip()!r}）")

        if op == "stage":
            plan["stage"] = int(args[0])
        elif op == "frames":
            plan["frames"] = int(args[0])
        elif op == "prefix":
            plan["prefix"] = safe_name_component(args[0], "prefix")  # A1 路径消毒
        elif op == "fire":
            a, b = args[0].split("-")
            plan["fire"] = (int(a), int(b))
        elif op == "bomb":
            if args[0] == "budget":
                plan["bomb_budget"] = int(args[1])
            elif args[0] == "at":
                plan["bomb_at"] = [int(x) for x in args[1].split(",")]
            else:
                err("bomb 用法: bomb budget <N> | bomb at <帧,帧,…>")
        elif op == "band":
            # band [<起帧>] x <lo>-<hi> y <lo>-<hi>  —— 多段走廊（生成期按帧切换 = C4 条件展开）
            toks2 = " ".join(args)
            start = 0
            m0 = re.match(r"(\d+)\s+", toks2)
            if m0:
                start = int(m0.group(1))
                toks2 = toks2[m0.end():]
            m = re.fullmatch(r"x\s+(\d+)-(\d+)\s+y\s+(\d+)-(\d+)", toks2)
            if not m:
                err("band 用法: band [<起帧>] x <lo>-<hi> y <lo>-<hi>")
            plan.setdefault("bands", []).append((start, tuple(int(g) for g in m.groups())))
        elif op == "sweep":
            if args[0] == "period":
                plan["sweep_period"] = int(args[1])
            elif args[0] == "phase":
                plan["sweep_phase"] = int(args[1])
            else:
                err("sweep 用法: sweep period <F> [phase <F>]")
        elif op == "capture_every":
            plan["capture_every"] = (int(args[0]), args[1])
        elif op == "when":
            # F1（复核 #78）：when 生成期展开档位**未实现** → 显式拒绝（fail-closed，不静默吞）。
            #   档位裁定：spec §1.2 承诺"生成期展开"但展开器不存在（旧版零消费者静默丢弃）；
            #   谓词 spawn_near/cluster_active/slot_count 求值语义未证（未证不发明），
            #   已验证的条件相位切换路径 = 多段 band <起帧>（v3 实测有效）→ 本轮取"拒绝未实现指令"档位。
            #   子句动作动词先行分诊：白名单外（含 poke 类）按各自语义拒绝，白名单内按 when 未实现拒绝。
            try:
                then_i = args.index("then")
            except ValueError:
                err("when 子句缺 then <action>（语法见 spec §1.2；实现档位=显式拒绝，复核 #78 F1）")
            if then_i + 1 >= len(args):
                err("when 子句 then 后缺动作动词（实现档位=显式拒绝，复核 #78 F1）")
            verb = args[then_i + 1]
            if verb in POKE_CLASS_VERBS:
                # 写注入伪装在 when 子句里同样拒绝（fail-closed 语义覆盖子句内部）
                err(f"生成器拒绝 when 子句内 {verb}：TAS 输入轨只允许按键（README 写注入纪律）"
                    f"——when 内嵌写注入不得静默吞（复核 #78 F1）")
            if verb not in WHEN_ACTION_WHITELIST:
                err(f"when 子句动作动词 {verb!r} 不在白名单 {WHEN_ACTION_WHITELIST}"
                    f"（未实现指令显式拒绝，复核 #78 F1）")
            err(f"when 生成期展开未实现（档位=显式拒绝，spec §1.2 更正注；复核 #78 F1）"
                f"：{verb} 动作不展开；条件相位切换请用多段 band <起帧>（已验证路径）")
        elif op in ("poke", "poke_every", "snapshot", "restore"):
            # fail-closed：正常通关场景禁 poke（README 纪律）；生成器无写注入能力
            err(f"生成器拒绝 {op}：TAS 输入轨只允许按键（README 写注入纪律）")
        else:
            err(f"未知计划指令 {op!r}")
    if plan["stage"] is None or plan["frames"] is None:
        raise PlanError(f"{path}: 缺 stage / frames")
    if plan["fire"] is None:
        raise PlanError(f"{path}: 缺 fire（R-tas-1：射击保持）")
    bands = plan.get("bands") or [(0, (40, 200, 60, 223))]
    for start, (x0, x1, y0, y1) in bands:
        if not (CLAMP_X[0] <= x0 < x1 <= CLAMP_X[1]) or not (CLAMP_Y[0] <= y0 < y1 <= CLAMP_Y[1]):
            raise PlanError(f"{path}: band 超出钳制域 X{CLAMP_X} Y{CLAMP_Y}（R-tas-3）")
    plan["bands"] = sorted(bands)
    plan["band"] = plan["bands"][0][1]
    return plan


# ---------------------------------------------------------------- 规则求解

def _triangle(t: float, period: float) -> float:
    """相位 t（帧）→ [-1,1] 三角波。"""
    p = (t % period) / period
    return 4 * p - 1 if p < 0.5 else 3 - 4 * p


def solve_movement(plan: dict) -> list[dict]:
    """R-tas-3/4/5：走廊内二维三角扫（不停留 = 不给定点接触窗口）。

    位置由已证速度对（R-tas-3：±2.8125 px/帧 + 钳制域）逐帧积分模拟；目标点 = x/y 三角波；
    输出 = RLE 后的 hold 段（纯按键）。启动点 (128,160) = gameplay-stage1 f1260 实测（N-0011/12）。
    """
    bands = plan.get("bands") or [(0, plan["band"])]
    def band_at(f):
        b = bands[0][1]
        for start, bb in bands:
            if f >= start:
                b = bb
        return b
    f_start, f_end = plan["fire"]
    x0, x1, y0, y1 = band_at(f_start)
    xm, xa = (x0 + x1) / 2, (x1 - x0) / 2
    ym, ya = (y0 + y1) / 2, (y1 - y0) / 2
    xper = max(4, plan["sweep_period"])
    yper = max(4, plan.get("v_sweep_period", xper * 5))
    phase = plan.get("sweep_phase", 0)
    px, py = 128.0, 160.0
    v = VEL_PX_PER_FRAME
    per_frame: list[tuple[int, tuple]] = []
    for f in range(f_start, f_end + 1):
        bx0, bx1, by0, by1 = band_at(f)
        if (bx0, bx1, by0, by1) != (x0, x1, y0, y1):
            x0, x1, y0, y1 = bx0, bx1, by0, by1
            xm, xa = (x0 + x1) / 2, (x1 - x0) / 2
            ym, ya = (y0 + y1) / 2, (y1 - y0) / 2
        tx = xm + xa * _triangle(f - f_start + phase, xper)
        ty = ym + ya * _triangle(f - f_start + phase, yper)
        btns = []
        if px < tx - 1.0:
            px = min(CLAMP_X[1], px + v)
            btns.append("right")
        elif px > tx + 1.0:
            px = max(CLAMP_X[0], px - v)
            btns.append("left")
        if py < ty - 1.0:
            py = min(CLAMP_Y[1], py + v)
            btns.append("down")
        elif py > ty + 1.0:
            py = max(CLAMP_Y[0], py - v)
            btns.append("up")
        per_frame.append((f, tuple(sorted(btns))))
    moves = []
    run_f, run_b = None, None
    for f, b in per_frame + [(f_end + 1, None)]:
        if b != run_b:
            if run_b:
                moves.append({"from": run_f, "to": f - 1, "buttons": list(run_b),
                              "rule": "R-tas-4 二维三角扫（不触钳）"})
            run_f, run_b = f, b
    return moves


def solve_bombs(plan: dict, timeline: list[dict]) -> list[int]:
    """R-tas-2：炸弹投放排程。显式 bomb at 优先；否则按调度流威胁窗密度取点。"""
    if plan["bomb_at"]:
        pts = sorted(set(plan["bomb_at"]))
    else:
        pts = []
        for rec in timeline:
            if len(rec["spawns"]) >= 1:
                f = int(tick_to_frame(rec["tick"]))
                if f > plan["fire"][0]:
                    pts.append(f)
        pts = sorted(set(pts))
    # 预算与投放去重（R-tas-2：在途窗内不重复投放）
    out, last = [], -10 ** 9
    cost = BOMB_CONSUME[0]
    budget = plan["bomb_budget"]
    for f in pts:
        if f < plan["fire"][0] or f > plan["frames"]:
            continue  # 预算外/超帧界（生成物必须落在 1..frames，scenario_run 校验）
        if f - last < BOMB_DEPLOY_DEDUP_HINT:
            continue
        if budget < cost:
            break
        out.append(f)
        last = f
        budget -= cost
    return out


# ---------------------------------------------------------------- 生成

def plan_to_scenario(plan: dict, timeline: list[dict]) -> tuple[str, dict]:
    moves = solve_movement(plan)
    bombs = solve_bombs(plan, timeline)
    f0, f1 = plan["fire"]
    L = []
    L.append("# 本文件由 tools/tas_gen.py 生成（勿手改）——B2 TAS 输入轨（纯按键，无 poke）。")
    L.append(f"# 规格：docs/spec/tas-input-dsl.md；plan: {plan['source']}")
    L.append("# 规则出处：R-tas-1 射击保持（B 长按，R-shot-gen-2 / shot-gating-f1 §1，A 级）；")
    L.append("#           R-tas-2 炸弹应急（A 边沿投放，R-fire-1..3；预算 $018F≤20）；")
    L.append("#           R-tas-3 走廊=钳制域子集（C-clamp-x/y，A 级）；")
    L.append("#           R-tas-4 横扫=经验启发式（C 级，明示非已证弹道）；")
    L.append("#           R-tas-5 避让 margin=命中包络候选12px+4px（hit-sampling-h16 §2，B 级候选）；")
    L.append("#           R-tas-6 弹速表语义未证（Wt-6）→ 不做弹道预测（显式不消费）；")
    L.append("#           R-tas-7 通关判定=$26 bit4 自然置位（bank00:8CFC，A 级）。")
    default_name = "tas-stg%02d-v1" % plan["stage"]
    L.append("name " + plan.get("name", default_name))
    L.append(f"frames {plan['frames']}")
    L.append("press_len 4")
    L.append("")
    if plan["prefix"]:
        L.append(f"# ===== 同源前缀（{plan['prefix']}，确定性对账：勿改）=====")
        src = SCENARIO_DIR / (plan["prefix"] + ".txt")
        fcut = plan["fire"][0] - 1   # 只同源**战斗段之前**的输入（标题/商店段）
        for line in src.read_text(encoding="utf-8").splitlines():
            s = line.split("#", 1)[0].strip()
            if s.startswith("press_len "):
                continue
            if not s.startswith(("press ", "hold ", "press_every ")):
                continue
            toks = s.split()
            if toks[0] == "press":
                rng = (int(toks[1]), int(toks[1]))
            else:  # hold / press_every：<起>-<止> 在参数 1（press_every 参数 0 是周期）
                a, b = toks[1 if toks[0] == "hold" else 2].split("-")
                rng = (int(a), int(b))
            if rng[0] > fcut:
                continue  # 战斗段输入由生成规则接管（防冲突/防重复 press）
            L.append(s)
        L.append("")
    L.append("# ===== 战斗段输入（生成规则 R-tas-*）=====")
    L.append(f"hold {f0}-{f1} b   # R-tas-1 主武器连发")
    for m in moves:
        for b in m["buttons"]:
            L.append(f"hold {m['from']}-{m['to']} {b}   # {m['rule']}")
    for bf in bombs:
        L.append(f"press {bf} a   # R-tas-2 炸弹（投放去重 {BOMB_DEPLOY_DEDUP_HINT}f）")
    L.append("")
    L.append("# ===== watch（轻量面：通关判定 + 输入纯净 + RNG/tick 探测）=====")
    for addr, label in (("0180", "stage_no"), ("0026", "flags26"), ("0027", "flags27"),
                        ("002f", "countdown2f"), ("002b", "rng_2b"), ("002c", "rng_2c"),
                        ("002d", "rng_2d"), ("0084", "tick_lo"), ("0085", "tick_hi"),
                        ("0300", "gate300"), ("0400", "pstate"), ("04e0", "player_x"),
                        ("04fc", "player_y"), ("018d", "lives"), ("018e", "money"),
                        ("018f", "bomb_ammo"), ("0018", "held")):
        L.append(f"watch {addr} {label}")
    L.append("")
    if plan.get("capture_every"):
        n, tag = plan["capture_every"]
        L.append(f"capture_every {n} {tag}")
    else:
        L.append("capture_every 1000 probe")
    L.append("capture 1260 combat-start")
    report = {
        "plan": plan,
        "moves": moves,
        "bombs": bombs,
        "timeline_projection": [
            {"tick": r["tick"], "frame_est": tick_to_frame(r["tick"]),
             "spawns": r["spawns"]} for r in timeline
        ],
        "assumptions": [
            "W-tas-3: tick→帧按实测锚插值 + 尾段 16f/tick 外推"
            "（tick558-815 实测 257 tick 全 16f；tick=16px 滚动进度已证=Wt-2 关闭，复核 #78 §4；"
            "外推仅覆盖 tick 816→1232 未实测尾段）",
            "R-tas-4 横扫为经验启发式（非已证弹道）",
            "R-tas-5 margin=16px 为包络候选+余量（W15 未定谳）",
            "Wt-6: 弹速表语义未闭环 → 不做弹道预测",
        ],
        # F1（复核 #78）：未实现 DSL 显式声明（档位=拒绝，不静默吞）
        "unimplemented_dsl": [
            "when <pred> then <action>：生成期展开档位未建 → 显式拒绝（spec §1.2 更正注，复核 #78 F1）；"
            "条件相位切换由多段 band <起帧> 承担（已验证路径）",
        ],
        "no_poke": True,
    }
    return "\n".join(L) + "\n", report


# ---------------------------------------------------------------- bot 输入日志 → 开环场景

def record_to_scenario(inputs_path: Path, name: str, frames: int | None = None) -> str:
    """C4 §5 bot 闭环：逐帧按键日志 → 开环 press/hold 轨（RLE）。"""
    rows = []
    for line in inputs_path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    if not rows:
        raise PlanError(f"{inputs_path}: 空输入日志")
    frames = frames or max(r["f"] for r in rows)
    by_btn: dict[str, list[tuple[int, bool]]] = {}
    for r in rows:
        for b in ("a", "b", "start", "select", "up", "down", "left", "right"):
            by_btn.setdefault(b, []).append((r["f"], bool(r.get(b))))
    L = [f"# bot 输入日志转写（tools/tas_gen.py record-to-scenario）→ 开环轨；"
         f"来源 {inputs_path.name}", f"name {name}", f"frames {frames}", "press_len 1", ""]
    for b, seq in by_btn.items():
        run = None
        for f, on in seq + [(frames + 1, False)]:
            if on and run is None:
                run = f
            elif not on and run is not None:
                L.append(f"hold {run}-{f - 1} {b}")
                run = None
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------- CLI

def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("gen", help="生成输入轨 scenario")
    g.add_argument("plan")
    g.add_argument("-o", "--out-dir", default=str(SCENARIO_DIR))
    g.add_argument("--name", default=None)
    t = sub.add_parser("timeline", help="打印调度流时间线 + tick→帧投影")
    t.add_argument("--stage", type=int, default=1)
    t.add_argument("--maxrec", type=int, default=200)
    r = sub.add_parser("report", help="生成报告 JSON（stdout）")
    r.add_argument("plan")
    c = sub.add_parser("record-to-scenario", help="bot 输入日志 → 开环场景")
    c.add_argument("inputs")
    c.add_argument("name")
    c.add_argument("-o", "--out-dir", default=str(SCENARIO_DIR))
    args = ap.parse_args()

    if args.cmd == "timeline":
        for rec in sched_timeline(args.stage, maxrec=args.maxrec):
            sp = " ".join(
                f"[T=${s['head']:02X} 成员${s['member_type']:02X}×{s['count']} 隔{s['interval_f']}f 参${s['param']:02X}]"
                for s in rec["spawns"])
            print(f"  +{rec['tick']:5d}t ≈ f{tick_to_frame(rec['tick']):8.0f}  "
                  f"@${rec['raw'].get('addr') or 0:04X} {sp}")
        return
    if args.cmd == "gen" or args.cmd == "report":
        try:
            plan = parse_plan(Path(args.plan))
            if args.cmd == "gen" and args.name:
                plan["name"] = safe_name_component(args.name, "--name")  # A1 路径消毒
            timeline = sched_timeline(plan["stage"])
            text, report = plan_to_scenario(plan, timeline)
        except PlanError as e:
            print(f"PlanError: {e}", file=sys.stderr)
            sys.exit(1)
        if args.cmd == "report":
            print(json.dumps(report, ensure_ascii=False, indent=2))
            return
        default_name = "tas-stg%02d-v1" % plan["stage"]
        out = Path(args.out_dir) / (plan.get("name", default_name) + ".txt")
        out.write_text(text, encoding="utf-8")
        rep = out.with_suffix(".report.json")
        rep.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
        print(f"生成 {out}（{len(text.splitlines())} 行）")
        print(f"报告 {rep}")
        return
    if args.cmd == "record-to-scenario":
        try:
            name = safe_name_component(args.name, "name")  # A1 路径消毒
            out = Path(args.out_dir) / f"{name}.txt"
            out.write_text(record_to_scenario(Path(args.inputs), name), encoding="utf-8")
        except PlanError as e:
            print(f"PlanError: {e}", file=sys.stderr)
            sys.exit(1)
        print(f"生成 {out}")


if __name__ == "__main__":
    main()
