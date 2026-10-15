#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/arena_discriminant.py —— 第四候选（受击保护/命中冷却）判别实验：构建 + 分析。

判别目标（arena-midboss §2.3 挂账收口；复核 #82 §6-1 方法）：
  中距（y 隔开，非贴身窗）单发点射 ×10 → 逐帧游戏侧写点（$064B HP / $05DC / $05F7 / $04DF / $04C）
  → 每发弹 HP 差分（0 或 −2）vs 时序。判据用**游戏侧写点**（模型框≠游戏命中框，f10068 教训）。

用法：
  python3 tools/arena_discriminant.py build-repro      # 旧 inputs → 开环复现场景 tas-arena-repro
  python3 tools/arena_discriminant.py build-shots      # 微调轨（中距单发 ×10）→ tas-arena-shots
  python3 tools/arena_discriminant.py analyze <run>    # 逐发判别表 + 判别字段时序 → analysis/traces/<run>.disc.json

纪律：纯按键轨（无 poke——场景解析期 fail-closed 由 scenario_run.py 保证）；
输入源 = .archive/tas-stg01-bot1-20261015-131847 的 10800 帧录制轨（12:39 前实施体）。
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import tas_gen  # noqa: E402  record-to-scenario 唯一实现

TRACES = ROOT / "analysis" / "traces"
SCEN = ROOT / "oracle" / "scenarios"
STALE_DIR = TRACES / ".archive" / "tas-stg01-bot1-20261015-131847"
STALE = STALE_DIR / "tas-stg01-bot1"
BOT_SCEN = SCEN / "tas-stg01-bot1.txt"

# 中距单发 ×10（B 点按 4f，间距 8f）：f10074..f10146，boss y≈119..168（stale 轨迹）→ dy≈47..96（y 隔开）
TAPS = [10074, 10082, 10090, 10098, 10106, 10114, 10122, 10130, 10138, 10146]
TAP_LEN = 4
CUT = 10070          # f≤CUT 原样保留（含 f10068 −2 伤害对照事件）

# 自机弹槽（tools/tas_bot.lua SHOT_SLOTS 观测活跃子集）：$80-$84=128-132、$9C-$A0=156-160
SLOT_GROUPS = [(0x80, 0x85), (0x9C, 0xA1)]

# 判别字段（复核 #82 §6-1 逐帧 watch 面）+ 弹槽逐帧观测（游戏侧消费时序，替代几何推断）
EXTRA_WATCH = [
    ("004c", "dmg_mode"),    # zp $4C 伤害模式旗（$D67C：0=一击死域，非 0 走减伤公式）
    ("04df", "boss_attr"),   # $04C4+27 属性掩码（$13 恒定候选；init 写 $33=相位/受击改写候选）
    ("05dc", "cnt_base"),    # $05DC 命中倒数基（slot0 视角；W18 语义挂账）
]
EXTRA_RANGES = []            # (from,to,label) 每帧 hex 串
for lo, hi in SLOT_GROUPS:
    tag = f"{lo:02x}"
    EXTRA_RANGES += [
        (f"{0x040E + lo:04x}", f"{0x040E + hi - 1:04x}", f"shot_state_{tag}"),  # state=$040E+Y
        (f"{0x04EE + lo:04x}", f"{0x04EE + hi - 1:04x}", f"shot_x_{tag}"),      # x=$04EE+Y
        (f"{0x055E + lo:04x}", f"{0x055E + hi - 1:04x}", f"shot_y_{tag}"),      # y=$055E+Y
        (f"{0x063E + lo:04x}", f"{0x063E + hi - 1:04x}", f"shot_tcode_{tag}"),  # 型码=$063E+Y
    ]
EXTRA_PROBE = ["064b", "05f7", "04df", "05dc", "004c"]   # 写点 PC（游戏侧注册/冷却/伤害路径）


def stale_watch_lines() -> list[str]:
    """对拍面同源：bot 场景 watch 行原样（28 标签），保证与 stale run 可逐帧对拍。"""
    out = []
    for raw in BOT_SCEN.read_text(encoding="utf-8").splitlines():
        s = raw.split("#", 1)[0].strip()
        if s.startswith("watch "):
            out.append(s)
    return out


def watch_block() -> list[str]:
    L = ["# ===== 对拍面（stale 同源 28 标签）+ 判别字段（复核 #82 §6-1：$064B/$05DC/$05F7/$04DF/$04C）====="]
    L += stale_watch_lines()
    L += [f"watch {a} {lab}" for a, lab in EXTRA_WATCH]
    L += [f"watch_range {a}-{b} {lab}" for a, b, lab in EXTRA_RANGES]
    L += ["", "# ===== 游戏侧写点探针（NMI 级写点 PC——判别用游戏侧注册，非几何框）====="]
    L += [f"probe_mem {a} {a}" for a in EXTRA_PROBE]
    L += ["", "capture_every 300 probe"]
    return L


def write_scenario(name: str, body: str, out_dir: Path) -> Path:
    p = out_dir / f"{name}.txt"
    p.write_text(body.rstrip("\n") + "\n", encoding="utf-8")
    return p


def build_repro() -> Path:
    """stale inputs 原样 → 开环轨（复现 arena 窗 f9916-10202）+ 判别 watch 面。"""
    inputs = STALE.with_suffix(".inputs.jsonl")
    if not inputs.is_file():
        raise SystemExit(f"缺 stale 输入日志: {inputs}")
    text = tas_gen.record_to_scenario(inputs, "tas-arena-repro")
    body = "\n".join([
        "# 第四候选判别实验 · 复现轨（record-to-scenario 开环）——",
        "#   来源：.archive/tas-stg01-bot1-20261015-131847/tas-stg01-bot1.inputs.jsonl（10800 帧，12:39 前实施体）",
        "#   目的：复现 arena 窗（f9916 中 boss 入场 / f10068 −2 / f10135-10202 贴身窗）+ 判别字段逐帧观测。",
        "#   与 stale run 的 28 对拍字段逐帧全等 = 复现成立（tools/arena_discriminant.py analyze）。",
        text.rstrip("\n"), "", *watch_block(), "",
    ])
    return write_scenario("tas-arena-repro", body, SCEN)


def build_shots() -> Path:
    """微调轨：f≤CUT 原样（含 f10068 −2 伤害对照）→ 中距单发 ×10（纯按键，无 poke）。"""
    rows = [json.loads(l) for l in STALE.with_suffix(".inputs.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    taps = {f for t in TAPS for f in range(t, t + TAP_LEN)}
    out_rows = []
    for r in rows:
        r = dict(r)
        if r["f"] > CUT:
            r["b"] = r["f"] in taps          # 其余键沿用 stale；B = 点按 ×10
        out_rows.append(r)
    inp_out = TRACES / "tas-arena-shots.inputs.jsonl"
    inp_out.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in out_rows), encoding="utf-8")
    text = tas_gen.record_to_scenario(inp_out, "tas-arena-shots")
    body = "\n".join([
        "# 第四候选判别实验 · 中距单发轨（纯按键，无 poke）——",
        f"#   f≤{CUT} 与 stale/复现轨逐帧全等（含 f10068 tcode=0 −2 伤害对照事件=同 run 内伤害路径正控）；",
        f"#   f>{CUT} 仅改 B：4f 点按 ×{len(TAPS)}（{TAPS[0]}..{TAPS[-1]}+{TAP_LEN - 1}，间距 8f）→ boss y≈119..168 中距（dy≈47..96，y 隔开）。",
        "#   判据=游戏侧写点逐帧（$064B/$05DC/$05F7/$04DF/$04C + 弹槽消费时序），不用几何框推断。",
        text.rstrip("\n"), "", *watch_block(), "",
    ])
    return write_scenario("tas-arena-shots", body, SCEN)


# ---------------------------------------------------------------- 分析（逐发判别表）

def load_frames(name: str) -> list[dict]:
    path = TRACES / f"{name}.frames.jsonl"
    return [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()]


def slot_series(rows: list[dict]) -> dict[int, dict[int, tuple[int, int, int, int]]]:
    """弹槽逐帧 (state,x,y,tcode)：从 watch_range hex 串还原。"""
    ser: dict[int, dict[int, tuple[int, int, int, int]]] = {}
    for lo, hi in SLOT_GROUPS:
        tag = f"{lo:02x}"
        for row in rows:
            w, f = row["w"], row["f"]
            st = bytes.fromhex(w[f"shot_state_{tag}"])
            xs = bytes.fromhex(w[f"shot_x_{tag}"])
            ys = bytes.fromhex(w[f"shot_y_{tag}"])
            tc = bytes.fromhex(w[f"shot_tcode_{tag}"])
            for i in range(hi - lo):
                yslot = lo + i
                ser.setdefault(yslot, {})[f] = (st[i], xs[i], ys[i], tc[i])
    return ser


def analyze(name: str) -> dict:
    rows = load_frames(name)
    byf = {r["f"]: r["w"] for r in rows}
    frames = [r["f"] for r in rows]
    ser = slot_series(rows)

    hp = {f: byf[f].get("boss_hp") for f in frames}
    hp_events = []
    for i in range(1, len(frames)):
        a, b = hp[frames[i - 1]], hp[frames[i]]
        if a is not None and b is not None and a != b:
            hp_events.append({"f": frames[i], "from": a, "to": b, "delta": int(a) - int(b)})

    shots = []          # 逐发（游戏侧槽占用起止）
    for yslot, s in ser.items():
        prev = 0
        spawn_f = None
        for f in frames:
            st, x, y, tc = s.get(f, (0, 0, 0, 0))
            if st != 0 and prev == 0:
                spawn_f, sx, sy = f, x, y
            if st == 0 and prev != 0 and spawn_f is not None:
                w = byf[f]
                dprev = hp.get(frames[frames.index(f) - 1]) if f in frames else None
                delta = (int(dprev) - int(hp[f])) if (dprev is not None and hp.get(f) is not None) else 0
                shots.append({
                    "yslot": yslot, "tcode": tc, "spawn_f": spawn_f, "die_f": f,
                    "spawn_xy": (sx, sy), "die_xy": (x, y),          # die_xy=末见帧位置（9px/帧运动滞后）
                    "boss_xy_at_die": (w.get("boss_x"), w.get("boss_y")),
                    "boss_hp_delta": delta,
                    "field_at_die": {k: w.get(k) for k in
                                     ("boss_hp", "boss_attr", "cnt_base", "dmg_mode", "boss_hitctr")},
                    "hp_events_same_frame": [e for e in hp_events if e["f"] == f],
                })
                spawn_f = None
            prev = st

    # 判别字段变化帧（全 run）：游戏侧写点可见性
    field_changes = {}
    for k in ("boss_attr", "cnt_base", "dmg_mode", "boss_hitctr"):
        ch, prev = [], None
        for f in frames:
            v = byf[f].get(k)
            if v != prev:
                ch.append({"f": f, "v": v})
                prev = v
        field_changes[k] = ch

    rep = {
        "run": name, "frames_total": len(frames),
        "hp_events": hp_events,
        "shots_self": [s for s in shots if s["tcode"] != 0x80],
        "shots_enemy_count": sum(1 for s in shots if s["tcode"] == 0x80),
        "field_changes": field_changes,
        "boss_track_sample": [
            {"f": f, "boss_xy": (byf[f].get("boss_x"), byf[f].get("boss_y")),
             "player_xy": (byf[f].get("player_x"), byf[f].get("player_y")),
             "hp": byf[f].get("boss_hp")}
            for f in frames if f % 10 == 0 and 9900 <= f <= 10250
        ],
    }
    out = TRACES / f"{name}.disc.json"
    out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    # 控制台：逐发判别表
    print(f"== analyze {name} == frames={len(frames)}  hp_events={len(hp_events)}")
    print("HP 差分事件: " + ", ".join(f"f{e['f']}({e['from']}→{e['to']})" for e in hp_events))
    print(f"{'弹':>4} {'tcode':>5} {'出生→消失':>14} {'落点':>12} {'boss@die':>12} {'ΔHP':>4}  判别字段@die")
    for s in rep["shots_self"]:
        fe = s["field_at_die"]
        print(f"{s['yslot']:>4} {s['tcode']:>5} f{s['spawn_f']}→f{s['die_f']:>5} "
              f"{str(s['die_xy']):>12} {str(s['boss_xy_at_die']):>12} {s['boss_hp_delta']:>4}  "
              f"attr={fe['boss_attr']} hitctr={fe['boss_hitctr']} cnt={fe['cnt_base']} dmg={fe['dmg_mode']}")
    print(f"报告: analysis/traces/{name}.disc.json")
    return rep


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build-repro", help="stale inputs → 开环复现场景")
    sub.add_parser("build-shots", help="中距单发 ×10 微调轨")
    a = sub.add_parser("analyze", help="逐发判别表")
    a.add_argument("run")
    args = ap.parse_args()
    if args.cmd == "build-repro":
        p = build_repro()
    elif args.cmd == "build-shots":
        p = build_shots()
    else:
        analyze(args.run)
        return 0
    print(f"生成 {p.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
