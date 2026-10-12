#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/diff_traces.py —— L3 帧级 trace 差分（trace-diff 门禁入口，S2→S5 试点）。

裁定面：game-core::player 帧 tick 序列的逐帧状态向量 == oracle probe trace（Mesen2 执行）。
对齐约定（940 帧 oracle 实证 0 失配）：`state(N) = tick(state(N-1), held(N))`——
输入当帧生效；帧计数 = 场景 DSL 的 inputPolled 序（tools/scenario_capture.lua）。

职责分工（无第二套扫描逻辑）：
- 本脚本：oracle 全量 trace → L3 golden 抽取（--extract）+ 编排；
- game-oracle 的 `l3-diff` bin：模拟执行 + 比对（模拟语义唯一在 game-core::player）。

用法：
    python3 tools/diff_traces.py --extract   # 从 analysis/traces/ 全量 trace 抽 golden（重跑场景用 scenario_run.py）
    python3 tools/diff_traces.py             # 跑 L3 首用例（默认 golden: oracle/traces/gameplay-stage1.player.jsonl）
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ORACLE_TRACE = ROOT / "analysis" / "traces" / "gameplay-stage1.frames.jsonl"
GOLDEN = ROOT / "oracle" / "traces" / "gameplay-stage1.player.jsonl"
WINDOW_START = 1260  # 实机段起点（各场景约定，见 oracle/scenarios/README.md）
# 死亡/重生窗口（t24/t31 取证，确定性 run）：受击判定/重生 = spec §4 未证域（H-16），禁止发明 ⇒
# 帧不入 L3 比对面；缺口即分段，l3-diff 重播种。游戏玩法段之外（boot/菜单）同样不比。
SCENES = {
    "gameplay-stage1": {
        "start": 1260,
        "skip": [(1831, 1916), (2047, 2101), (2131, 2175)],
        "fields": "named",  # 命名 watch（player_x 等）
    },
    "weapon-pickup": {
        "start": 1260,
        "skip": [(1721, 1789), (1956, 2084)],
        "fields": "iram",   # iram 区间解码（$0300-$07FF blob）
    },
}

# 命名 watch 源（gameplay-stage1）
NAMED = {"player_x": "x", "player_y": "y", "player_x_subpix": "sx", "player_y_subpix": "sy",
         "controller_held": "held"}
# iram blob 源（weapon-pickup）：地址 → 字段（held 取 zp[0x18]）
IRAM = {0x04E0: "x", 0x04FC: "y", 0x0470: "sx", 0x048C: "sy"}


def fields_of(w: dict, kind: str) -> dict | None:
    """从一行 watch 集提取状态向量字段；缺源返回 None。"""
    if kind == "named":
        if any(k not in w for k in NAMED):
            return None
        return {out: w[src] for src, out in NAMED.items()}
    blob = bytes.fromhex(w["iram"])
    zp = bytes.fromhex(w["zp"])
    out = {"held": zp[0x18]}
    for addr, name in IRAM.items():
        out[name] = blob[addr - 0x300]
    return out


def extract_weapon_events(scene: str) -> int:
    """武器/弹药事件 golden（kind=weapon: $34/$33 转移；kind=bomb: $0190/$018F 转移）。

    R-bomb-2 断言源（t38 FA-3）：$0190 切换与 $018F 变化须同帧发生（三方同步的机器侧两变量）。
    注意 $018F/$0190 在 $0100 页（栈页游戏变量区）——须场景 watch 显式覆盖，勿跨页误映射。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.weapons.jsonl"
    if not src.is_file():
        print(f"oracle 全量 trace 缺失：{src}", file=sys.stderr)
        return 2
    events = []
    prev_wpn = None
    prev_bomb = None
    warned = False
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        w = rec["w"]
        zp = bytes.fromhex(w["zp"])
        wpn = (zp[0x34], zp[0x33])  # (weapon_slot, weapon_level) —— $0034/$0033 零页
        if "bomb_ammo" in w and "bomb_type_index" in w:
            bomb = (w["bomb_type_index"], w["bomb_ammo"], zp[0x3B])  # $0190/$018F（$0100 页）+ $3B 查表索引
        else:
            bomb = None
            if not warned:
                print(f"  注意：{scene} 缺 bomb_ammo/bomb_type_index watch（$0100 页），bomb 事件不入 golden")
                warned = True
        if prev_wpn is not None:
            if wpn != prev_wpn:
                events.append({"f": rec["f"], "kind": "weapon",
                               "before": {"slot": prev_wpn[0], "level": prev_wpn[1]},
                               "after": {"slot": wpn[0], "level": wpn[1]}})
            if bomb is not None and prev_bomb is not None and bomb != prev_bomb:
                events.append({"f": rec["f"], "kind": "bomb",
                               "before": {"type": prev_bomb[0], "ammo": prev_bomb[1], "idx": prev_bomb[2]},
                               "after": {"type": bomb[0], "ammo": bomb[1], "idx": bomb[2]}})
        prev_wpn = wpn
        if bomb is not None:
            prev_bomb = bomb
    dst.parent.mkdir(parents=True, exist_ok=True)
    with open(dst, "w", encoding="utf-8") as f:
        for e in events:
            f.write(json.dumps(e) + "\n")
    n_bomb = sum(1 for e in events if e["kind"] == "bomb")
    print(f"武器事件抽取：{scene} 共 {len(events)} 事件（weapon {len(events) - n_bomb} / bomb {n_bomb}）→ {dst.relative_to(ROOT)}")
    return 0



# 弹对象 SoA 字段（t32 $70 间距网格；watch_range 0400-06ff 的分列解释）
SHOT_FIELDS = [("lifetime", 0x0E), ("script_ptr", 0x7E), ("pos_x", 0xEE),
               ("pos_y", 0x15E), ("motion_code", 0x1CE), ("shot_type", 0x23E)]
SHOT_SLOTS = 0x70  # 字段间距 = 槽容量（slot 步长 1 字节）
SHOT_SAMPLE_N = 4  # 帧降采样（声明口径，可复现）


def extract_shot_trajectories(scene: str) -> int:
    """弹对象轨迹 golden（分列命名：lifetime/script_ptr/pos_x/pos_y/motion_code/shot_type）。

    采样口径（显式声明，可复现）：slot 全覆盖（0..0x6F）；每个存活窗口（lifetime≠0 连续段）
    首帧+每 SHOT_SAMPLE_N 帧+末帧入 golden；出生帧=种子，比对=逐帧 tick（R-shot-motion-1/2）；
    死亡帧（lifetime 归 0）为未证域边界不入比对面。数据源=shot_objs watch blob（$0400-$06FF）
    按数组分列解析——为 enemy 轨迹（$1C 网格）留结构。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.shots.jsonl"
    if not src.is_file():
        print(f"oracle 全量 trace 缺失：{src}", file=sys.stderr)
        return 2
    # 逐 slot 收集存活窗口
    segments: dict[int, list[dict]] = {}
    open_seg: dict[int, list] = {}
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        blob_w = rec["w"].get("shot_objs")
        if blob_w is None:
            print(f"  {scene} 缺 shot_objs watch（$0400-06ff）", file=sys.stderr)
            return 2
        blob = bytes.fromhex(blob_w)
        f = rec["f"]
        for slot in range(SHOT_SLOTS):
            fields = {name: blob[off + slot] for name, off in SHOT_FIELDS}
            alive = fields["lifetime"] != 0
            if alive:
                open_seg.setdefault(slot, []).append({"f": f, "slot": slot, **fields})
            elif slot in open_seg:
                segments.setdefault(slot, []).append(open_seg.pop(slot))
    for slot, seg in open_seg.items():
        segments.setdefault(slot, []).append(seg)
    # 帧降采样 + 落盘
    n_rows = 0
    with open(dst, "w", encoding="utf-8") as out:
        for slot in sorted(segments):
            for seg in segments[slot]:
                keep = [i for i in range(len(seg)) if i % SHOT_SAMPLE_N == 0]
                if len(seg) - 1 not in keep:
                    keep.append(len(seg) - 1)  # 末帧必留
                for i in keep:
                    out.write(json.dumps(seg[i]) + "\n")
                    n_rows += 1
    n_seg = sum(len(v) for v in segments.values())
    print(f"弹轨迹抽取：{scene} {n_seg} 个存活窗口 / {n_rows} 行（slot 全覆盖，帧降采样 n={SHOT_SAMPLE_N}）→ {dst.relative_to(ROOT)}")
    return 0

def extract() -> int:
    rc = 0
    for scene, cfg in SCENES.items():
        src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
        if not src.is_file():
            print(f"oracle 全量 trace 缺失：{src}（先 tools/scenario_run.py run {scene}）", file=sys.stderr)
            return 2
        rows = []
        for line in src.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            f = rec["f"]
            if f < cfg["start"] or any(lo <= f <= hi for lo, hi in cfg["skip"]):
                continue
            fields = fields_of(rec["w"], cfg["fields"])
            if fields is None:
                continue
            rows.append({"f": f, **fields})
        if len(rows) < 2:
            print(f"{scene}: 窗口样本不足（{len(rows)}）", file=sys.stderr)
            return 2
        dst = ROOT / "oracle" / "traces" / f"{scene}.player.jsonl"
        with open(dst, "w", encoding="utf-8") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")
        print(f"player golden 抽取：{scene} {len(rows)} 帧 → {dst.relative_to(ROOT)}")
        rc = rc or extract_weapon_events(scene)
    return rc


def run_diff() -> int:
    goldens = (sorted((ROOT / "oracle" / "traces").glob("*.player.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.weapons.jsonl")))
    if not goldens:
        print(f"L3 golden 缺失（先 --extract）", file=sys.stderr)
        return 2
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "game-oracle", "--bin", "l3-diff", "--"]
        + [str(g) for g in goldens],
        cwd=ROOT,
    )
    return proc.returncode


def main() -> int:
    if "--shots-extract" in sys.argv:
        # 【实验性】弹轨迹抽取：槽位几何未证（见 oracle/traces/README t45 发现——
        # t32 $70 间距仅锚定字段基址，槽位步长/容量/分配器 $888D 属 B5/A5 未证域；
        # 命中数据含跨系统串扰迹象，不得作为 golden 入库）——decomp-analyst 取证后启用。
        return extract_shot_trajectories("gameplay-stage1")
    if "--extract" in sys.argv:
        rc = extract()
        return rc if rc else run_diff()
    return run_diff()


if __name__ == "__main__":
    sys.exit(main())
