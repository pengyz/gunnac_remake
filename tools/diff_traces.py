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
# 槽位偏移集（t48 定谳 docs/findings/shot-slot-geometry.md：4 组×5 槽，组距 $1C）
SHOT_SLOTS = [0x80, 0x81, 0x82, 0x83, 0x84,
              0x9C, 0x9D, 0x9E, 0x9F, 0xA0,
              0xB8, 0xB9, 0xBA, 0xBB, 0xBC,
              0xD4, 0xD5, 0xD6, 0xD7, 0xD8]
SHOT_SAMPLE_N = 4  # 帧降采样（声明口径，可复现）


def _lifetime_walk_possible(lt_prev: int, gap: int, durs=frozenset({0xFF, 0x0A, 0x05, 0x01, 0x04})) -> set:
    """t52 F2b 寿命倒数模型的非确定行走：v>1 递减；v==1 时装载时长表值（refill）。
    返回 gap 帧后可能的寿命集合（v==0=释放，不出现在存活窗口内）。"""
    poss = {lt_prev}
    for _ in range(gap):
        nxt = set()
        for v in poss:
            if v > 1:
                nxt.add(v - 1)
            elif v == 1:
                nxt.update(durs)
        poss = nxt
    return poss


def _spawn_signature_ok(rows: list, player_at: dict) -> bool:
    """R-shot-gen-3 生成链 A 级签名：窗口起点坐标 = 玩家坐标 + 一步（|Δ|≤9，LUT 幅值界）。
    W7 网格驻留值（$1C 交错区）不满足 ⇒ 不是弹窗口（负例场景 0 窗口的判定源）。"""
    r0 = rows[0]
    px, py = player_at[r0["f"]]
    return abs(r0["pos_x"] - px) <= 9 and abs(r0["pos_y"] - py) <= 9


def _is_shot_window(rows: list) -> bool:
    """窗口资格判定（t52 F2b）：整窗逐段符合寿命倒数模型 ⇒ 弹窗口；
    $1C 网格驻留值（静态/递增）不满足 ⇒ 排除（W7 交错残余，未证所有权不入库）。"""
    for prev, cur in zip(rows, rows[1:]):
        gap = cur["f"] - prev["f"]
        if gap <= 0:
            return False
        if cur["lifetime"] not in _lifetime_walk_possible(prev["lifetime"], gap):
            return False
        if cur["shot_type"] > 5:  # R-shot-gen-3 生成链：弹型=X（0..5）
            return False
        if cur["motion_code"] > 0x12:  # C-action-stream 观测值域上界（$890B 区最大 0x12）
            return False
    return all(r["shot_type"] <= 5 and r["motion_code"] <= 0x12 for r in rows[:1])


def _pos_jump_gt_lut(prev: dict, cur: dict, gap: int) -> bool:
    """粗可观测边界：单帧位置变化超 LUT 幅值界（|Δ|≤9×gap，C-motion-lut-x/y A 级界）
    ⇒ 同槽死亡→重生合窗切开。仅用粗界切分，精确逐值由 L3 重放对拍（非空转）。"""
    dx = abs(cur["pos_x"] - prev["pos_x"])
    dy = abs(cur["pos_y"] - prev["pos_y"])
    return dx > 9 * gap or dy > 9 * gap


def extract_shot_trajectories(scene: str) -> int:
    """弹对象轨迹/slot state 向量 golden（分列命名六数组；slot 全覆盖）。

    采样口径（显式声明，可复现，t55 转正）：
    ①slot 全覆盖 = t48 定谳 20 槽偏移集（$80-$84/$9C-$A0/$B8-$BC/$D4-$D8），state==0 空槽；
    ②窗口资格 = t52 F2b 寿命倒数语义（逐段非确定行走可及）——网格驻留值窗口排除（W7）；
    ③每个合格窗口 首帧 + 每 SHOT_SAMPLE_N 帧 + 末帧入 golden；
    ④出生帧（合格窗口起点）补全 20 槽行（首次适应分配断言的当帧状态向量源）；
    ⑤比对 = 出生帧播种 + 逐帧 tick（R-shot-motion-1/2）+ 寿命递减断言 + 分配断言；
    死亡帧（state 归 0）为未证域边界不入轨迹比对面。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.shots.jsonl"
    if not src.is_file():
        print(f"oracle 全量 trace 缺失：{src}", file=sys.stderr)
        return 2
    frames: list[tuple[int, dict]] = []
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        w = rec["w"]
        blob_w = w.get("iram")
        if blob_w is None:
            print(f"  {scene} 无 iram watch——跳过弹轨迹抽取")
            return 0
        blob = bytes.fromhex(blob_w)
        base = 0x300
        slots = {slot: {name: blob[(0x400 + off + slot) - base] for name, off in SHOT_FIELDS}
                 for slot in SHOT_SLOTS}
        frames.append((rec["f"], slots))
    # 逐槽收集连续存活窗口并做资格判定；win=槽内弹生命周期序号（跨窗重播种标识）
    emit: dict[tuple[int, int], int] = {}
    win_of: dict[tuple[int, int], int] = {}
    n_qualified = 0
    player_at = {rec_f: (slots_pl.get("_px", 0), slots_pl.get("_py", 0)) for rec_f, slots_pl in []}
    player_at = {}
    for f, slots in frames:
        pass
    # 玩家坐标源 = 命名 watch（两场景齐备）
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        w = rec["w"]
        if "player_x" in w and "player_y" in w:
            player_at[rec["f"]] = (w["player_x"], w["player_y"])

    def flush(window: list, slot: int) -> None:
        nonlocal n_qualified
        if (len(window) >= 2 and _is_shot_window(window)
                and player_at.get(window[0]["f"]) is not None
                and _spawn_signature_ok(window, player_at)):
            n_qualified += 1
            win_id = n_qualified
            for i, r in enumerate(window):
                win_of[(r["f"], slot)] = win_id
                if i % SHOT_SAMPLE_N == 0 or i == len(window) - 1:
                    emit[(r["f"], slot)] = win_id
            for s2 in SHOT_SLOTS:
                emit.setdefault((window[0]["f"], s2), 0)
                emit.setdefault((window[0]["f"] - 1, s2), 0)  # 出生前帧全向量（分配断言真值源）

    for slot in SHOT_SLOTS:
        window: list[dict] = []
        for f, slots in frames:
            fl = slots[slot]
            row = {"f": f, "slot": slot, **fl}
            if fl["lifetime"] == 0:
                flush(window, slot)
                window = []
            elif window:
                prev = window[-1]
                gap = f - prev["f"]
                if (gap <= 0 or _pos_jump_gt_lut(prev, row, gap)
                        or row["lifetime"] not in _lifetime_walk_possible(prev["lifetime"], gap)):
                    flush(window, slot)  # 合窗切开（死亡→重生/网格切换）
                    window = [row]
                else:
                    window.append(row)
            else:
                window = [row]
        flush(window, slot)
    if n_qualified == 0:
        if dst.exists():
            dst.unlink()  # 清除陈旧 golden（负例场景不得留旧档）
        print(f"弹轨迹抽取：{scene} 0 个合格弹窗口（负例段 ✓ 无 B 长按静默/W7 驻留值全排除）——不产出 golden")
        return 0
    n_rows = 0
    with open(dst, "w", encoding="utf-8") as out:
        for f, slots in frames:
            for slot in SHOT_SLOTS:
                if (f, slot) in emit:
                    win = win_of.get((f, slot), 0)
                    out.write(json.dumps({"f": f, "slot": slot, "win": win, **slots[slot]}) + "\n")
                    n_rows += 1
    print(f"弹轨迹抽取：{scene} {n_qualified} 个合格弹窗口 / {n_rows} 行（20 槽全覆盖，帧降采样 n={SHOT_SAMPLE_N} + 出生帧全向量）→ {dst.relative_to(ROOT)}")
    return 0



def extract_enemy_churn(scene: str) -> int:
    """敌对象 churn golden（R-enemy-6 生成/活动段；$1C 网格 28 槽）。

    采样口径（声明）：帧降采样 n=8，逐行 = {f, state[28], x[28], y[28]}（hex 串分列）。
    比对面：R-enemy-4 坐标钳（活跃槽 x∈$0E-$F3 / y∈$14-$DF，A 级）+ churn 计数轨迹（快照）。
    命中/清除段（H-16）未证不入面；双 SoA 桥（W12）未证——本 golden 以 $1C 网格观测为准。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.enemy.jsonl"
    if not src.is_file():
        return 2
    n = 0
    wrote = False
    with open(dst, "w", encoding="utf-8") as out:
        for line in src.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            w = rec["w"]
            blob_w = w.get("iram")
            if blob_w is None:
                if not wrote:
                    print(f"  {scene} 无 iram watch——跳过 enemy churn 抽取")
                return 0
            if rec["f"] % 8 != 0 or rec["f"] < WINDOW_START:
                continue  # 游戏段起（boot/菜单网格未初始化，钳规则证据域之外）
            blob = bytes.fromhex(blob_w)
            state = blob[0x100:0x100 + 28]   # $0400 状态字段
            xs = blob[0x1E0:0x1E0 + 28]      # $04E0 x 坐标
            ys = blob[0x1FC:0x1FC + 28]      # $04FC y 坐标
            out.write(json.dumps({
                "f": rec["f"],
                "state": state.hex(), "x": xs.hex(), "y": ys.hex(),
            }) + "\n")
            wrote = True
            n += 1
    print(f"enemy churn 抽取：{scene} {n} 行（帧降采样 n=8，28 槽分列）→ {dst.relative_to(ROOT)}")
    return 0



def extract_stage_scroll(scene: str) -> int:
    """关卡卷轴 $1C/$1D 逐帧向量 golden（R-bg-scroll-1..3，L3 全等对拍）。

    采样口径（声明）：f1193-2400 **逐帧全采**（无降采样——规则为帧级确定性值序列）。
    载体 = zp watch（$1C/$1D 逐帧向量，L3 状态向量已含 zp）。UI 卷轴通道（$AB3B）分立
    不在此面（R-bg-scroll-4）。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.scroll.jsonl"
    if not src.is_file():
        return 2
    n = 0
    with open(dst, "w", encoding="utf-8") as out:
        for line in src.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            f = rec["f"]
            if f < 1193 or f > 2400:
                continue
            w = rec["w"]
            zp_w = w.get("zp")
            if zp_w is None:
                print(f"  {scene} 无 zp watch——跳过卷轴抽取")
                return 0
            zp = bytes.fromhex(zp_w)
            out.write(json.dumps({"f": f, "scroll_x": zp[0x1C], "scroll_y": zp[0x1D]}) + "\n")
            n += 1
    print(f"卷轴向量抽取：{scene} {n} 行（f1193-2400 逐帧全采）→ {dst.relative_to(ROOT)}")
    return 0



def extract_money_events(scene: str) -> int:
    """金钱转移事件 golden（R-money-4/5：$E0-$E6 逐帧转移）。
    口径（声明）：全部转移帧逐事件记录 {f, before(7 位), after(7 位)}——事件驱动无降采样。
    断言面：每转移 = 单次加法器事件（delta=加数表值）或封顶（$E0-$E5 全 9）。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.money.jsonl"
    if not src.is_file():
        return 2
    events = 0
    prev = None
    with open(dst, "w", encoding="utf-8") as out:
        for line in src.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            w = rec["w"]
            zp_w = w.get("zp")
            if zp_w is None:
                return 0
            zp = bytes.fromhex(zp_w)
            cur = list(zp[0xE0:0xE7])
            if prev is not None and cur != prev:
                out.write(json.dumps({"f": rec["f"], "before": prev, "after": cur}) + "\n")
                events += 1
            prev = cur
    print(f"金钱事件抽取：{scene} {events} 事件 → {dst.relative_to(ROOT)}")
    return 0



TAG_TO_MODE = {
    "boot-black": "Black", "title": "Title",
    "menu-540": "Menu", "menu-720": "Menu", "menu-900": "Menu", "menu-end": "Menu",
    "shop-arrival": "Shop", "shop-page": "ShopInfo", "exit-transition": "Transition",
    "stage-start": "StageStart", "mid-combat": "Combat",
}


def extract_flow_sequence(scene: str) -> int:
    """流程模式序列 golden（R-ui-6/7，场景 capture 标签锚定）。
    口径（声明）：全部非 probe 标签逐事件记录 {f, mode}——事件驱动全采；end=序列终止标记不入行。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.captures.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.flow.jsonl"
    if not src.is_file():
        return 0
    rows = []
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        mode = TAG_TO_MODE.get(rec.get("tag", ""))
        if mode is None:
            continue
        rows.append({"f": rec["f"], "mode": mode})
    if not rows:
        print(f"流程序列：{scene} 无模式标签——不产出 golden")
        return 0
    with open(dst, "w", encoding="utf-8") as out:
        for r in rows:
            out.write(json.dumps(r) + "\n")
    print(f"流程序列抽取：{scene} {len(rows)} 事件 → {dst.relative_to(ROOT)}")
    return 0



def extract_appearances(scene: str) -> int:
    """敌出现/tick 事件 golden（t86 硬断言面）。
    判据（t83 正寻址激活）：**$0400+槽 state 0→非0**（112 事件/86 tick 帧实证）。
    口径（声明）：原始 trace 全分辨率事件驱动全采；比对面 = R-stage-tick-1（tick 网格）
    + R-stage-tick-3（每 tick 事件数分布）+ 位置/顺序（golden 逐值）；记录集 x 对齐为
    W12 桥观测（VM $A4→$04xx 写链未证）。
    """
    src = ROOT / "analysis" / "traces" / f"{scene}.frames.jsonl"
    dst = ROOT / "oracle" / "traces" / f"{scene}.stage-appear.jsonl"
    if not src.is_file():
        return 0
    events = []
    prev = None
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        if rec["f"] < 1260:
            continue
        w = rec["w"]
        blob_w = w.get("iram")
        if blob_w is None:
            return 0
        blob = bytes.fromhex(blob_w)
        cur = list(blob[0x100:0x11C])  # $0400 状态字段（正寻址 28 槽）
        if prev is not None:
            for i in range(28):
                if cur[i] != 0 and prev[i] == 0:
                    events.append({"f": rec["f"], "slot": i,
                                   "x": blob[0x1E0 + i], "y": blob[0x1FC + i]})
        prev = cur
    with open(dst, "w", encoding="utf-8") as out:
        for e in events:
            out.write(json.dumps(e) + "\n")
    rec_x = [175, 123, 139, 119, 119, 117, 117, 128]
    match = sum(1 for e in events if e["x"] in rec_x)
    print(f"tick 事件抽取：{scene} {len(events)} 事件（记录集 x 对齐 {match}——W12 观测）→ {dst.relative_to(ROOT)}")
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
    for scene in ("fire-hold", "gameplay-stage1", "weapon-pickup"):
        rc = rc or extract_shot_trajectories(scene)
        rc = rc or extract_enemy_churn(scene)
    # 卷轴 golden 仅 gameplay-stage1（R-bg-scroll 相位/边界帧系该场景时间线定谳；
    # 他场景时间线未证不抽取）
    rc = rc or extract_stage_scroll("gameplay-stage1")
    for scene in ("gameplay-stage1", "shop"):
        rc = rc or extract_money_events(scene)
    for scene in ("gameplay-stage1", "shop", "respawn-window", "title-menu"):
        rc = rc or extract_flow_sequence(scene)
    rc = rc or extract_appearances("gameplay-stage1")
    return rc


def run_diff() -> int:
    goldens = (sorted((ROOT / "oracle" / "traces").glob("*.player.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.weapons.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.shots.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.enemy.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.scroll.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.money.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.flow.jsonl")) +
        sorted((ROOT / "oracle" / "traces").glob("*.stage-appear.jsonl")))
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
    if "--extract" in sys.argv:
        rc = extract()
        return rc if rc else run_diff()
    return run_diff()


if __name__ == "__main__":
    sys.exit(main())
