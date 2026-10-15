#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/tas_bot.py —— B2 v4 bot 闭环驱动（record-to-scenario 闭环 + 状态对拍 + 逐发归因）。

闭环口径（B2 §7 教训；判据 arena-midboss §5）：
  bot 实时决策（tools/tas_bot.lua，只读 RAM→按键）→ 录制逐帧按键 inputs.jsonl
  → tas_gen record-to-scenario 转写**纯 press/hold 开环轨**（生成物无条件指令/无 poke）
  → scenario_run 回放 → 与 bot run 的逐帧 watch 全字段**状态对拍**（两遍一致才算闭环成立）。

用法：
  python3 tools/tas_bot.py closure <bot_scenario> [--replay NAME] [--skip-run]
      ① 读 bot run 产物（inputs/events/trace）
      ② 生成回放场景 oracle/scenarios/<replay>.txt（record-to-scenario + 同源 watch/probe 面
         + 锚帧 capture）
      ③ 跑回放（scenario_run.run_scenario）
      ④ 逐帧状态对拍 → analysis/traces/<bot>.closure.json（verdict=pass 仅当 0 mismatch）
  python3 tools/tas_bot.py compare <runA> <runB>      # 通用两 trace 逐帧对拍
  python3 tools/tas_bot.py attrib <run> [--json]      # 逐发归因表 + 第四候选（受击保护/冷却）数据
  python3 tools/tas_bot.py anchors <run> [--json]     # 关末/商店/关 2 锚帧提取（golden 回填用）

纪律：本工具不写内存、不改 trace；只做生成/对拍/归因（生成物 = 纯按键场景 + 报告 JSON）。
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import tas_gen  # noqa: E402  record-to-scenario 唯一实现
from tools import scenario_run as sr  # noqa: E402  DSL 唯一解析器 + 驱动

TRACES = ROOT / "analysis" / "traces"
SCEN = ROOT / "oracle" / "scenarios"

MISMATCH_SAMPLE = 40      # 报告中逐条 mismatch 样例上限
ATTRIB_SAMPLE = 200       # 逐发归因表行数上限（全量计数不受限）


# ---------------------------------------------------------------- trace 读取/对拍

def load_trace(name: str) -> tuple[list[int], list[dict]]:
    """读 <name>.frames.jsonl → (帧号列表, 逐帧 watch 字典列表)。"""
    path = TRACES / f"{name}.frames.jsonl"
    if not path.is_file():
        raise SystemError(f"缺 trace：{path}")
    frames: list[int] = []
    watches: list[dict] = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            frames.append(row["f"])
            watches.append(row["w"])
    return frames, watches


def compare_traces(name_a: str, name_b: str) -> dict:
    """两 run 逐帧 watch 全字段对拍。verdict=pass 仅当帧集与全字段逐帧全等。"""
    fa, wa = load_trace(name_a)
    fb, wb = load_trace(name_b)
    n = min(len(fa), len(fb))
    mismatches: list[dict] = []
    field_counts: Counter = Counter()
    total = 0
    for i in range(n):
        if fa[i] != fb[i]:
            total += 1
            field_counts["__frame_index__"] += 1
            if len(mismatches) < MISMATCH_SAMPLE:
                mismatches.append({"f_a": fa[i], "f_b": fb[i], "field": "__frame_index__"})
            continue
        keys = set(wa[i]) | set(wb[i])
        for k in sorted(keys):
            va, vb = wa[i].get(k, "<missing>"), wb[i].get(k, "<missing>")
            if va != vb:
                total += 1
                field_counts[k] += 1
                if len(mismatches) < MISMATCH_SAMPLE:
                    mismatches.append({"f": fa[i], "field": k, "a": va, "b": vb})
    return {
        "run_a": name_a,
        "run_b": name_b,
        "frames_a": len(fa),
        "frames_b": len(fb),
        "frames_compared": n,
        "frame_count_equal": len(fa) == len(fb),
        "mismatch_total": total,
        "mismatch_by_field": dict(field_counts),
        "mismatch_sample": mismatches,
        "verdict": "pass" if (total == 0 and len(fa) == len(fb)) else "fail",
    }


# ---------------------------------------------------------------- 回放场景生成

def extract_watch_block(bot_scenario: Path) -> list[str]:
    """从 bot 场景抽 watch/watch_range/probe_mem 行（对拍面与探针面同源）。"""
    out = []
    for raw in bot_scenario.read_text(encoding="utf-8").splitlines():
        s = raw.split("#", 1)[0].strip()
        if s.startswith(("watch ", "watch_range ", "probe_mem ")):
            out.append(s)
    return out


def build_replay(bot_scenario: Path, bot_name: str, replay_name: str,
                 anchors: dict, capture_every: int = 300) -> Path:
    """inputs.jsonl → 开环场景（record-to-scenario）+ 同源 watch/probe + 锚帧 capture。"""
    tas_gen.safe_name_component(replay_name, "replay name")
    inputs = TRACES / f"{bot_name}.inputs.jsonl"
    if not inputs.is_file():
        raise SystemError(f"缺录制输入：{inputs}")
    text = tas_gen.record_to_scenario(inputs, replay_name)
    lines = [text.rstrip("\n"), "",
             "# ===== 对拍/探针面（与 bot 场景同源，closure 对拍逐帧全字段）====="]
    lines += extract_watch_block(bot_scenario)
    lines += ["", "# ===== 锚帧采集（自然 trigger 证据；帧号来自 bot run trace）====="]
    lines.append(f"capture_every {capture_every} probe")
    for key, a in anchors.items():
        if a and a.get("capture"):
            lines.append(f"capture {a['f']} {a['tag']}")
    lines.append("")
    out = SCEN / f"{replay_name}.txt"
    out.write_text("\n".join(lines), encoding="utf-8")
    return out


# ---------------------------------------------------------------- 锚帧提取

def _bit(v: int, b: int) -> bool:
    return (int(v) >> b) & 1 == 1


def detect_anchors(name: str) -> dict:
    """从 run trace 提取关 1 自然通关链锚帧（golden 回填；机器可重导）。

    判定式与 G-S5-6 injected 锚同构（anchors 台账 predicate 字段），触发=natural：
      clear_gate  : $26 bit4 0→1（f>2000 排除菜单→开 stage 过渡窗 f913-1192）
      stg01-clear : flags27 bit7（E309 一次性入场）∧ countdown2f==50（E312）同帧
      flyout      : countdown2f==0（E342 飞离向量写对 $041C:=$A4/$0438:=$0D）
      gate300     : $0300≠0 ∧ 同帧 $26 bit4 清（E366→E36E）
      inc180      : $0180 1→2（E37C INC $180）
      stg02-shop  : stage_no==2 ∧ pstate==0（$0400:=0 商店 init，AD65-1）
      stg02-start : 关始引入序列 $0188 递减窗内（「地区-2」同口径采样）
      stg02-combat: shop 后 +784（gs5-stg01-clear-anchor 同谱系 offset）
    """
    frames, watches = load_trace(name)
    by_f = dict(zip(frames, watches))

    def w(i, label):
        return watches[i].get(label)

    labels = set(watches[0]) if watches else set()
    need = {"flags26", "flags27", "countdown2f", "gate300", "stage_no", "pstate"}
    if not need <= labels:
        raise SystemError(f"{name}: watch 面缺 {sorted(need - labels)}（锚帧提取需对拍面同源）")

    a = {k: None for k in ("clear_gate", "stg01-clear", "flyout", "gate300-open",
                          "inc-180", "stg02-shop", "stg02-exit", "stg02-start", "stg02-combat")}

    def first(pred, lo=0, hi=None):
        hi = len(frames) if hi is None else hi
        for i in range(lo, hi):
            if pred(i):
                return frames[i], i
        return None, None

    # clear_gate：bit4 0→1
    fg, ig = first(lambda i: i > 0 and frames[i] > 2000
                   and _bit(w(i, "flags26"), 4) and not _bit(w(i - 1, "flags26"), 4))
    if fg:
        a["clear_gate"] = {"f": fg, "tag": "gate-26-bit4", "capture": True,
                           "predicate": "$26 bit4 自然置位（0→1，f>2000）；上游=波次记录 2 完成 "
                                        "（bank00:8CFC 消费 $CA bit0，A 级）"}
        fc, ic = first(lambda i: i >= ig and _bit(w(i, "flags27"), 7)
                       and w(i, "countdown2f") == 50, ig)
        if fc:
            a["stg01-clear"] = {"f": fc, "tag": "clear-entry", "capture": True,
                                "predicate": "flags27 bit7 置位(E309 一次性入场) ∧ countdown2f==50"
                                             "(E312 倒数初始化) 同帧；trigger=自然（$26 bit4 自然置位链）"}
        ff, ifz = first(lambda i: i >= ig and w(i, "countdown2f") == 0, ig)
        if ff:
            a["flyout"] = {"f": ff, "tag": "flyout-vector", "capture": True,
                           "predicate": "倒数尽（countdown2f==0）同帧 E342 飞离向量写对 "
                                        "$041C:=$A4/$0438:=$0D"}
        f3, i3 = first(lambda i: i >= ig and w(i, "gate300") != 0, ig)
        if f3:
            a["gate300-open"] = {"f": f3, "tag": "clear-dispatch", "capture": True,
                                 "predicate": "$0300≠0 帧（E366 推进闸）"}
        f2, i2 = first(lambda i: i >= ig and w(i, "stage_no") == 2, ig)
        if f2:
            a["inc-180"] = {"f": f2, "tag": "stage-advance", "capture": True,
                            "predicate": "$0180=2 写帧（E37C INC $180）"}
        fs, ishop = first(lambda i: i >= ig and w(i, "stage_no") == 2 and w(i, "pstate") == 0, ig)
        if fs:
            a["stg02-shop"] = {"f": fs, "tag": "shop-init", "capture": True,
                               "predicate": "stage_no==2 ∧ pstate==0（$0400:=0 商店 init，"
                                            "bank15:E16B far-call 00 01 C3 9B=$9BC3）"}
            a["stg02-exit"] = {"f": fs + 284, "tag": "exit-fade4", "capture": True,
                               "predicate": "出店近黑过渡（光标「4.退」+A 后 30f 淡出窗；"
                                            "dark_frac 口径同 gs5-stg01-clear-anchor stg02-exit）"}
            a["stg02-start"] = {"f": fs + 334, "tag": "stg02-start", "capture": True,
                                "predicate": "「地区-2」采样帧（关始引入序列 $0188 63→0 窗内；"
                                             "offset 同 gs5-stg01-clear-anchor shop→start 334f）"}
            a["stg02-combat"] = {"f": fs + 784, "tag": "stg02-combat", "capture": True,
                                 "predicate": "战斗段采样帧（自机+敌弹；offset 同 "
                                              "gs5-stg01-clear-anchor shop→combat 784f）"}
    return {"run": name, "anchors": a, "frames_total": len(frames),
            "clear_gate_f": fg, "stage_no_final": watches[-1].get("stage_no") if watches else None}


# ---------------------------------------------------------------- 逐发归因

def _boss_box(form: int) -> list[tuple[int, int, int, int]]:
    """形 → 框组（tools/tas_bot.lua BOX_BY_FORM 同源；A 级框表 bank02:$97BA/$9812）。"""
    table = {
        0x02: [(-2, 2, -4, 4)], 0x0B: [(-4, 4, -4, 4)], 0x21: [(-2, 2, -6, 6)],
        0x22: [(-6, 6, -2, 2)], 0x28: [(-4, 4, -4, 4)], 0x0C: [(-7, 7, -7, 7)],
        0x33: [(-12, 12, -8, 8), (-8, 8, -12, 12)],
    }
    return table.get(form, [(-10, 10, -10, 10)])


def _overlaps_boss(sx: int, sy: int, bx: int, by: int, form: int) -> bool:
    """弹框（7×10：x−4..x+3, y−5..y+5）× boss 框组（形表偏移叠加 boss 坐标）相交判定。"""
    sl, sr_, st, sb = sx - 4, sx + 3, sy - 5, sy + 5
    for (l, r, t, b) in _boss_box(form):
        bl, br, bt, bb = bx + l, bx + r, by + t, by + b
        if not (sr_ < bl or br < sl or sb < bt or bb < st):
            return True
    return False


def attribute_shots(name: str) -> dict:
    """逐发归因（判据 2/第四候选数据面）：每发自机弹 = 出生帧/落点 × 消失帧/落点
    × boss 框相交 × HP 差分（trace 逐帧 boss_hp）。"""
    frames, watches = load_trace(name)
    idx = {f: i for i, f in enumerate(frames)}
    ev_path = TRACES / f"{name}.events.jsonl"
    events = [json.loads(l) for l in open(ev_path, encoding="utf-8") if l.strip()] \
        if ev_path.is_file() else []
    hp = {f: w.get("boss_hp") for f, w in zip(frames, watches)}
    bx = {f: w.get("boss_x") for f, w in zip(frames, watches)}
    by = {f: w.get("boss_y") for f, w in zip(frames, watches)}
    bf = {f: w.get("boss_form") for f, w in zip(frames, watches)}
    bt = {f: w.get("boss_type") for f, w in zip(frames, watches)}

    spawns: dict[tuple, dict] = {}
    rows: list[dict] = []
    for e in events:
        if e["kind"] == "shot_spawn" and e.get("owner", 0) == 0:
            spawns[(e["yslot"], e["f"])] = e
        elif e["kind"] == "shot_die" and e.get("owner", 0) == 0:
            # 找最近同槽出生
            key = None
            for (ys, sf), sp in spawns.items():
                if ys == e["yslot"] and sf <= e["f"]:
                    if key is None or sf > key[1]:
                        key = (ys, sf)
            sp = spawns.pop(key, None) if key else None
            f0 = sp["f"] if sp else None
            f1 = e["f"]
            prev_f = frames[idx[f1] - 1] if idx.get(f1) and idx[f1] > 0 else None
            delta = 0
            if prev_f is not None and hp.get(prev_f) is not None and hp.get(f1) is not None:
                delta = int(hp[prev_f]) - int(hp[f1])
            form = int(bf.get(f1) or 0)
            boss_hit = (bt.get(f1) not in (None, 0) and
                        _overlaps_boss(e["x"], e["y"],
                                       int(bx.get(f1) or 0), int(by.get(f1) or 0), form))
            offscreen = e["x"] <= 2 or e["x"] >= 252 or e["y"] <= 2 or e["y"] >= 250
            if boss_hit and delta >= 2:
                cls = "boss_confirmed"
            elif boss_hit and delta <= 0:
                cls = "boss_no_damage"      # ★ 第四候选（受击保护/冷却）数据点
            elif offscreen:
                cls = "offscreen"
            elif e.get("hit_slot", -1) >= 0:
                cls = f"object_slot{e['hit_slot']}"
            else:
                cls = "unknown"
            rows.append({
                "spawn_f": f0, "die_f": f1,
                "spawn_xy": (sp["x"], sp["y"]) if sp else None,
                "die_xy": (e["x"], e["y"]),
                "boss_xy_at_die": (bx.get(f1), by.get(f1)),
                "boss_hp_delta": delta, "class": cls,
            })
    counts = Counter(r["class"] for r in rows)
    hp_deltas = [(e["f"], e.get("delta")) for e in events
                 if e["kind"] == "hp_delta" and (e.get("to") == 0 or abs(e.get("delta", 0)) >= 2)]
    boss_hits = [r for r in rows if r["class"] == "boss_confirmed"]
    # 复核 #82 F3-2：HP 差分 ≥2 事件（帧级重导）× boss_confirmed 配对 ⇒ 未归因面
    # （模型框近似/一帧滞后使多数真实命中落 unknown/offscreen——不披露会被读成"仅 N 次命中"）。
    confirmed_die_frames = {r["die_f"] for r in boss_hits}
    hp_ge2_frames: list[int] = []
    for i in range(1, len(frames)):
        a, b = hp.get(frames[i - 1]), hp.get(frames[i])
        if a is not None and b is not None and int(a) - int(b) >= 2:
            hp_ge2_frames.append(frames[i])
    hp_ge2_unattributed = [f for f in hp_ge2_frames if f not in confirmed_die_frames]
    return {
        "run": name,
        "shots_self_total": len(rows),
        "class_counts": dict(counts),
        "boss_hits_confirmed": len(boss_hits),
        "boss_no_damage": counts.get("boss_no_damage", 0),
        "hp_delta_events": len(hp_deltas),
        "hp_delta_ge2_events": len(hp_ge2_frames),
        "hp_delta_ge2_unattributed": len(hp_ge2_unattributed),
        "hp_delta_ge2_unattributed_frames": hp_ge2_unattributed,
        "rows_sample": rows[:ATTRIB_SAMPLE],
        "fourth_candidate_note": (
            "boss_no_damage=0 且 boss 框相交弹逐一对应 HP−2 ⇒ 受击保护/命中冷却（第四候选）排除；"
            "boss_no_damage>0 ⇒ 逐条列出（坐实第四候选需同框相交+0 差分+无死亡门转位）"
        ),
    }


# ---------------------------------------------------------------- CLI

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("closure", help="录制→回放→状态对拍（闭环）")
    c.add_argument("bot_scenario")
    c.add_argument("--replay", default=None, help="回放场景名（默认 <bot>-replay）")
    c.add_argument("--skip-run", action="store_true", help="只生成回放场景不跑")
    p = sub.add_parser("compare", help="两 run 逐帧对拍")
    p.add_argument("run_a")
    p.add_argument("run_b")
    t = sub.add_parser("attrib", help="逐发归因 + 第四候选数据")
    t.add_argument("run")
    t.add_argument("--json", action="store_true")
    n = sub.add_parser("anchors", help="锚帧提取（golden 回填）")
    n.add_argument("run")
    n.add_argument("--json", action="store_true")
    args = ap.parse_args()

    if args.cmd == "closure":
        spath = sr.scenario_path_of(args.bot_scenario)
        plan = sr.parse_scenario(spath)
        bot = plan["name"]
        if not plan.get("bot"):
            print(f"closure 需要 bot 场景（{spath} 无 bot 指令）", file=sys.stderr)
            return 2
        replay = args.replay or (bot + "-replay")
        anch = detect_anchors(bot)
        rp = build_replay(spath, bot, replay, anch["anchors"])
        print(f"回放场景: {sr.relp(rp)}（anchors={sum(1 for v in anch['anchors'].values() if v)}）")
        if args.skip_run:
            return 0
        man = sr.run_scenario(str(rp))
        cmp_report = compare_traces(bot, replay)
        rep = {
            "bot_run": bot, "replay_run": replay,
            "replay_scenario": sr.relp(rp),
            "anchors": anch,
            "closure_compare": cmp_report,
            "replay_manifest_done": man.get("done"),
            "verdict": "pass" if cmp_report["verdict"] == "pass" else "fail",
        }
        out = TRACES / f"{bot}.closure.json"
        out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({k: rep[k] for k in ("bot_run", "replay_run", "verdict")},
                         ensure_ascii=False))
        print(f"closure mismatch={cmp_report['mismatch_total']} "
              f"frames={cmp_report['frames_compared']} → {sr.relp(out)}")
        return 0 if rep["verdict"] == "pass" else 1

    if args.cmd == "compare":
        rep = compare_traces(args.run_a, args.run_b)
        print(json.dumps(rep, ensure_ascii=False, indent=2))
        return 0 if rep["verdict"] == "pass" else 1

    if args.cmd == "attrib":
        rep = attribute_shots(args.run)
        if args.json:
            print(json.dumps(rep, ensure_ascii=False, indent=2))
        else:
            print(f"run={rep['run']} 自机弹 {rep['shots_self_total']} 发；分类={rep['class_counts']}")
            print(f"命中确认（HP−2 逐发）: {rep['boss_hits_confirmed']}；"
                  f"boss 框相交 0 差分: {rep['boss_no_damage']}；HP 差分事件 {rep['hp_delta_events']}")
            print(f"HP 差分 ≥2 事件（帧级）: {rep['hp_delta_ge2_events']}；"
                  f"未归因（hp_delta_ge2_unattributed）: {rep['hp_delta_ge2_unattributed']} "
                  f"@ {rep['hp_delta_ge2_unattributed_frames']}")
            for r in rep["rows_sample"]:
                print(f"  f{r['spawn_f']}→f{r['die_f']} spawn{r['spawn_xy']} die{r['die_xy']} "
                      f"ΔHP={r['boss_hp_delta']} {r['class']}")
        return 0

    if args.cmd == "anchors":
        rep = detect_anchors(args.run)
        if args.json:
            print(json.dumps(rep, ensure_ascii=False, indent=2))
        else:
            for k, v in rep["anchors"].items():
                print(f"  {k:<14} {('f' + str(v['f'])) if v else '—':>8}  "
                      f"{v['tag'] if v else ''}  {v['predicate'] if v else ''}")
        return 0
    return 2


if __name__ == "__main__":
    sys.exit(main())
