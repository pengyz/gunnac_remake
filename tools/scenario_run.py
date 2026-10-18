#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/scenario_run.py —— S2 oracle 场景采集框架（驱动 + DSL 唯一解析器）。

职责：
1. 解析场景 DSL（oracle/scenarios/*.txt）——**DSL 语义只在本文件解析**（无第二套解析器）；
2. 生成 analysis/_gen/<name>.lua = PLAN 表字面量 + tools/scenario_capture.lua 执行器；
3. 驱动 Mesen2 2.1.1 采集：xvfb-run -a tools/mesen2/Mesen rom/gunnac_cn.nes <lua> /DoNotSaveSettings
   （装载用法已验证；emu.stop 不退进程 ⇒ 驱动等 DONE 标记后 SIGTERM 收尾）；
4. 校验产物并落 run 清单（可复现证据）。

用法：
    python3 tools/scenario_run.py list
    python3 tools/scenario_run.py gen <name>            # 只生成 lua（调试）
    python3 tools/scenario_run.py run <name|路径> [--timeout 秒]
    python3 tools/scenario_run.py run-all [--timeout 秒]

帧计数约定（与 tools/scenario_capture.lua 一致，差分复现靠它锁死）：
- frame 在 inputPolled 回调递增：第 N 次 poll = 帧 N，schedule[N] 的按键在该帧注入；
- press <f> <btn> = 帧 f..f+press_len-1 按住（press_len 默认 2），之后释放；
- hold <a-b> <btn> = 帧 a..b（含）按住，帧 b+1 释放；
- endFrame（帧 N 渲染完成后）写 trace 行 / 截图 / RAM dump；frame >= frames 或 bot 早停时写 DONE
  并停机（task-105：bot 跳帧/分叉可 overshoot ⇒ DONE 记实际帧；task-110：完成判据以 DONE 为准）。

输出（均可用 --out/--shots 覆盖，默认 analysis/ 与 shots/）：
    analysis/traces/<name>.frames.jsonl    逐帧状态向量（w=watch 字节/range hex，可选 ram 全量）
    analysis/traces/<name>.captures.jsonl  采集点索引（截图路径 + RAM hex 文件 + cpu 寄存器）
    analysis/traces/<name>_fNNNNN_<tag>.ram.hex  采集点全量 2KB RAM（hex，4096 字符）
    shots/<name>_fNNNNN_<tag>.png          截图（emu.takeScreenshot）
    analysis/traces/<name>.run.json        运行清单（sha256 + 计数 + 路径）
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

try:  # POSIX（Linux 原生 / WSL）
    import fcntl
except ImportError:  # Windows: headless（testRunner）驱动模式
    fcntl = None  # type: ignore[assignment]

ROOT = Path(__file__).resolve().parent.parent


def _abs(env: str, default: str) -> Path:
    v = os.environ.get(env)
    q = Path(v) if v else Path(default)
    return q if q.is_absolute() else (ROOT / q)


# Mesen2 可执行：优先 GUNNAC_MESEN 覆盖；否则按宿主平台选 Linux 无扩展名版 / Windows .exe 版。
if os.environ.get("GUNNAC_MESEN"):
    MESEN = _abs("GUNNAC_MESEN", "tools/mesen2/Mesen")
elif os.name == "nt":
    MESEN = ROOT / "tools" / "mesen2" / "Mesen.exe"
else:
    MESEN = ROOT / "tools" / "mesen2" / "Mesen"
# headless：`--testRunner <lua> <rom>`（无窗口）。**2026-10-18 实测：testRunner 模式下
# emu.setInput 不生效**（getInput 读回为空、$18 恒 0、游戏停在开场）——有输入的场景必须用 GUI 模式。
# 因此 Windows 默认 **GUI（窗口）模式**；无输入场景可用 GUNNAC_HEADLESS=1 强制 headless（更快）。
HEADLESS = os.environ.get("GUNNAC_HEADLESS", "0") == "1"
# Windows GUI 模式：无需 Xvfb/DISPLAY；脚本以位置参数传入并 AutoStartScriptOnLoad。
GUI_WINDOWS = (not HEADLESS) and os.name == "nt"
ROM = _abs("GUNNAC_ROM", "rom/gunnac_cn.nes")
TRACES_DIR = _abs("GUNNAC_TRACES", "analysis/traces")
SHOTS_DIR = _abs("GUNNAC_SHOTS", "shots")
HARNESS = ROOT / "tools" / "scenario_capture.lua"
SCENARIO_DIR = ROOT / "oracle" / "scenarios"
GEN_DIR = ROOT / "analysis" / "_gen"

BUTTONS = {"a", "b", "start", "select", "up", "down", "left", "right"}
PRESS_LEN_DEFAULT = 2
# B2 v4 bot 闭环（tools/tas_bot.lua 决策器）：策略白名单 fail-closed（未知策略拒绝）。
# bot 场景 = 实时决策（读 RAM→按键）+ 逐帧按键录制（inputs.jsonl）→ tas_gen record-to-scenario
# 转写开环轨回放复核（闭环两遍一致）。**bot 场景禁 poke**（无写注入——通关必须自然）。
BOT_POLICIES = {"stg01-clear", "stage-clear"}  # stage-clear=八关参数化策略（通关令步骤 1-2）
BOT_LUA = ROOT / "tools" / "tas_bot.lua"


class ScenarioError(Exception):
    """DSL 解析/校验错误（带 文件:行号）。"""


def pick_display() -> str:
    """选空闲 X display（:99 起，按锁文件/socket 跳过占用）。"""
    for n in range(99, 200):
        if not Path(f"/tmp/.X{n}-lock").exists() and not Path(f"/tmp/.X11-unix/X{n}").exists():
            return f":{n}"
    raise ScenarioError("无空闲 X display（/tmp/.X11-unix 全占）")


def wait_for_x(display: str, xvfb: subprocess.Popen, timeout_s: float = 10.0) -> None:
    """等 X socket 就绪；Xvfb 起不来即报错（fail-closed）。"""
    sock = Path(f"/tmp/.X11-unix/X{display.lstrip(':')}")
    t0 = time.time()
    while not sock.exists():
        if xvfb.poll() is not None:
            raise ScenarioError(f"Xvfb {display} 启动失败（exit {xvfb.returncode}）")
        if time.time() - t0 > timeout_s:
            raise ScenarioError(f"Xvfb {display} socket 超时未就绪")
        time.sleep(0.1)


def relp(path: Path) -> str:
    """ROOT 内路径显示为相对 POSIX；ROOT 外（如 /tmp 探索场景）原样显示。"""
    try:
        return Path(path).resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


# ---------------------------------------------------------------- DSL 解析

def parse_scenario(path: Path) -> dict:
    """解析场景 DSL → PLAN dict。所有语义判定唯一入口。"""
    path = Path(path)
    plan: dict = {
        "name": path.stem,
        "frames": None,
        "press_len": PRESS_LEN_DEFAULT,
        "presses": [],
        "holds": [],
        "captures": [],
        "capture_every": None,
        "watch": [],
        "watch_ranges": [],
        "ramframe": False,
        "probe_mem": [],
        "pokes": [],
        "bot": None,
    }
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        toks = line.split()
        op, args = toks[0], toks[1:]

        def err(msg: str) -> ScenarioError:
            return ScenarioError(f"{path}:{lineno}: {msg}（行: {raw.strip()!r}）")

        if op == "name":
            if len(args) != 1:
                raise err("name 需要 1 个参数")
            plan["name"] = args[0]
        elif op == "frames":
            if len(args) != 1 or not args[0].isdigit() or int(args[0]) <= 0:
                raise err("frames 需要正整数")
            plan["frames"] = int(args[0])
        elif op == "press_len":
            if len(args) != 1 or not args[0].isdigit() or int(args[0]) <= 0:
                raise err("press_len 需要正整数")
            plan["press_len"] = int(args[0])
        elif op == "press":
            if len(args) < 2 or not args[0].isdigit():
                raise err("press 用法: press <帧号> <按钮...>")
            btns = [b.lower() for b in args[1:]]
            for b in btns:
                if b not in BUTTONS:
                    raise err(f"未知按钮 {b!r}（可选: {sorted(BUTTONS)}）")
            plan["presses"].append({"frame": int(args[0]), "buttons": btns})
        elif op == "press_every":
            # press_every <N> <起>-<止> <按钮...>：每 N 帧点按一次（解析期展开为 presses，无新执行语义）
            if len(args) < 3 or not args[0].isdigit() or int(args[0]) <= 0 or "-" not in args[1]:
                raise err("press_every 用法: press_every <N> <起>-<止> <按钮...>")
            a_s, b_s = args[1].split("-", 1)
            if not (a_s.isdigit() and b_s.isdigit()) or int(a_s) > int(b_s):
                raise err("press_every 区间必须是 起<=止 的非负整数")
            btns = [b.lower() for b in args[2:]]
            for b in btns:
                if b not in BUTTONS:
                    raise err(f"未知按钮 {b!r}（可选: {sorted(BUTTONS)}）")
            for f in range(int(a_s), int(b_s) + 1, int(args[0])):
                plan["presses"].append({"frame": f, "buttons": btns})
        elif op in ("poke", "poke_every"):
            # poke <帧> <hex地址> <hex值>：帧 f inputPolled 时写 CPU 总线（memType=8）一字节。
            # poke_every <N> <起>-<止> <hex地址> <hex值>：每 N 帧写一次（解析期展开为 pokes）。
            # 用途（G-S5-6 触发注入取证）：**外部事件注入**（如 $CA bit0=流末记录近端上游）；
            # 注入是证据链一部分，产出锚必须标注 trigger=injected（不得冒充自然可达）。
            def _poke_check(addr_s: str, val_s: str):
                addr = addr_s.lower()
                if len(addr) != 4 or any(c not in "0123456789abcdef" for c in addr):
                    raise err(f"poke 地址须为 4 位 hex，实际 {addr_s!r}")
                val = val_s.lower()
                if len(val) != 2 or any(c not in "0123456789abcdef" for c in val):
                    raise err(f"poke 值须为 2 位 hex，实际 {val_s!r}")
                return addr, int(val, 16)
            if op == "poke":
                if len(args) != 3 or not args[0].isdigit():
                    raise err("poke 用法: poke <帧号> <hex地址> <hex值>")
                addr, val = _poke_check(args[1], args[2])
                plan["pokes"].append({"frame": int(args[0]), "addr": addr, "val": val})
            else:
                if len(args) != 4 or not args[0].isdigit() or "-" not in args[1]:
                    raise err("poke_every 用法: poke_every <N> <起>-<止> <hex地址> <hex值>")
                a_s, b_s = args[1].split("-", 1)
                if not (a_s.isdigit() and b_s.isdigit()) or int(a_s) > int(b_s):
                    raise err("poke_every 区间必须是 起<=止 的非负整数")
                addr, val = _poke_check(args[2], args[3])
                for f in range(int(a_s), int(b_s) + 1, int(args[0])):
                    plan["pokes"].append({"frame": f, "addr": addr, "val": val})
        elif op == "hold":
            if len(args) < 2 or "-" not in args[0]:
                raise err("hold 用法: hold <起>-<止> <按钮...>")
            a_s, b_s = args[0].split("-", 1)
            if not (a_s.isdigit() and b_s.isdigit()) or int(a_s) > int(b_s):
                raise err("hold 区间必须是 起<=止 的非负整数")
            btns = [b.lower() for b in args[1:]]
            for b in btns:
                if b not in BUTTONS:
                    raise err(f"未知按钮 {b!r}（可选: {sorted(BUTTONS)}）")
            plan["holds"].append({"from": int(a_s), "to": int(b_s), "buttons": btns})
        elif op == "capture":
            if len(args) != 2 or not args[0].isdigit():
                raise err("capture 用法: capture <帧号> <tag>")
            plan["captures"].append({"frame": int(args[0]), "tag": args[1]})
        elif op == "capture_every":
            if len(args) != 2 or not args[0].isdigit() or int(args[0]) <= 0:
                raise err("capture_every 用法: capture_every <N> <tag>")
            plan["capture_every"] = {"n": int(args[0]), "tag": args[1]}
        elif op == "watch":
            # watch <hex 地址> [标签] —— 标签缺省=地址本身（语义命名必须有台账证据，见 AGENTS.md §3）
            if len(args) not in (1, 2):
                raise err("watch 用法: watch <hex 地址> [标签]")
            addr = args[0].lower()
            if len(addr) != 4 or any(c not in "0123456789abcdef" for c in addr):
                raise err(f"地址须为 4 位 hex（如 0320），实际 {args[0]!r}")
            plan["watch"].append({"addr": addr, "label": args[1] if len(args) == 2 else addr})
        elif op == "probe_mem":
            if len(args) != 2:
                raise err("probe_mem 用法: probe_mem <lo> <hi>（16 进制，NMI 级写点插桩）")
            plan["probe_mem"].append([args[0], args[1]])
        elif op == "watch_range":
            # watch_range <hex 起>-<hex 止> [标签]
            if len(args) not in (1, 2) or "-" not in args[0]:
                raise err("watch_range 用法: watch_range <hex起>-<hex止> [标签]")
            a_s, b_s = (s.lower() for s in args[0].split("-", 1))
            if not (len(a_s) == 4 and len(b_s) == 4):
                raise err("watch_range 地址须为 4 位 hex")
            if int(a_s, 16) > int(b_s, 16):
                raise err("watch_range 起点必须 <= 止点")
            label = args[1] if len(args) == 2 else f"{a_s}-{b_s}"
            plan["watch_ranges"].append({"from": a_s, "to": b_s, "label": label})
        elif op == "assist-mode":
            # 辅助声明（2026-10-16 不死跑到底+战略备胎）：显式标注辅助口径（双层标注 run 级）；
            # 声明后 poke 白名单扩 $0180 选关（备胎场景专用——bot 全程跑不声明不跳关）。
            if len(args) != 1:
                raise err("assist-mode 用法: assist-mode <reason>")
            plan["assist_mode"] = args[0]
        elif op == "ramframe":
            if len(args) != 1 or args[0] not in ("on", "off"):
                raise err("ramframe 用法: ramframe on|off")
            plan["ramframe"] = args[0] == "on"
        elif op == "bot":
            # B2 v4：bot <policy> from <帧> —— 实时决策器从帧 N 接管输入（录制逐帧按键 →
            # record-to-scenario 闭环）。策略白名单 fail-closed（tools/tas_bot.lua）。
            if len(args) != 3 or args[1] != "from" or not args[2].isdigit():
                raise err("bot 用法: bot <policy> from <帧>")
            if args[0] not in BOT_POLICIES:
                raise err(f"未知 bot 策略 {args[0]!r}（白名单 {sorted(BOT_POLICIES)}）")
            if plan["bot"] is not None:
                raise err("bot 指令重复")
            plan["bot"] = {"policy": args[0], "from": int(args[2])}
        else:
            raise err(f"未知 DSL 指令 {op!r}")

    if plan["frames"] is None:
        raise ScenarioError(f"{path}: 缺 frames 指令")
    frames = plan["frames"]
    seen = set()
    for e in plan["presses"]:
        if not 1 <= e["frame"] <= frames:
            raise ScenarioError(f"{path}: press 帧 {e['frame']} 超出 1..{frames}")
        key = (e["frame"], tuple(e["buttons"]))
        if key in seen:
            raise ScenarioError(f"{path}: 重复 press {key}")
        seen.add(key)
    for e in plan["holds"]:
        if not 1 <= e["from"] <= e["to"] <= frames:
            raise ScenarioError(f"{path}: hold 区间 {e['from']}-{e['to']} 超出 1..{frames}")
    for e in plan["captures"]:
        if not 1 <= e["frame"] <= frames:
            raise ScenarioError(f"{path}: capture 帧 {e['frame']} 超出 1..{frames}")
    for e in plan["pokes"]:
        if not 1 <= e["frame"] <= frames:
            raise ScenarioError(f"{path}: poke 帧 {e['frame']} 超出 1..{frames}")
    if plan["bot"] is not None:
        # 纪律锁：bot 场景禁 poke（决策器无写注入能力；通关必须自然，README 写注入纪律）
        # 【2026-10-16 用户裁定·不死跑到底】秘籍三址白名单特许（游戏自身机制，D4 干净）：
        # $01B6=WEAPON 行 / $01B7=BOMBER 行 / $01BE=DEATH·NON DEATH（≠0=不死）——
        # community-absorption-audit T13/336（A/B 级）；除此之外 poke 仍禁（$26 门不碰）。
        CHEAT_FLAG_POKE = {"01b6", "01b7", "01be"}
        allowed = CHEAT_FLAG_POKE | ({"0180"} if plan.get("assist_mode") else set())
        bad = [e for e in plan["pokes"] if e["addr"] not in allowed]
        if bad:
            raise ScenarioError(f"{path}: bot 场景禁 poke（仅秘籍三址白名单特许；越界 {bad}）")
        if not 1 <= plan["bot"]["from"] <= frames:
            raise ScenarioError(f"{path}: bot 起始帧 {plan['bot']['from']} 超出 1..{frames}")
        if not BOT_LUA.is_file():
            raise ScenarioError(f"{path}: bot 策略文件缺失 {BOT_LUA}")
    return plan


# ---------------------------------------------------------------- Lua 生成

def lua_str(s: str) -> str:
    out = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\r", "\\r")
    return f'"{out}"'


def to_lua(v) -> str:
    if v is None:
        return "nil"   # PLAN.bot 缺省（B2 v4）：Lua nil 语义
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        return lua_str(v)
    if isinstance(v, list):
        return "{" + ",".join(to_lua(x) for x in v) + "}"
    if isinstance(v, dict):
        return "{" + ",".join(f"[{lua_str(k)}]={to_lua(val)}" for k, val in v.items()) + "}"
    raise TypeError(f"不可序列化为 Lua: {v!r}")


def generate_lua(plan: dict, scenario_path: Path) -> tuple[str, str]:
    """返回 (lua 源码, lua sha256)。PLAN 前置 + 执行器整拼（无 dofile 依赖）。"""
    plan_lua = dict(plan)
    if plan_lua.get("bot") is None:
        plan_lua.pop("bot", None)   # 无 bot 的场景 PLAN 字节不变（历史 lua sha 锁不受影响）
    plan_lua["out_dir"] = str((TRACES_DIR).resolve())
    plan_lua["shots_dir"] = str((SHOTS_DIR).resolve())
    harness = HARNESS.read_text(encoding="utf-8")
    bot_src = ""
    if plan.get("bot"):
        # B2 v4：决策器源码整拼（无 dofile 依赖；PLAN 后、执行器前）
        bot_src = "\n-- ===== bot 决策器（tools/tas_bot.lua，policy=%s）=====\n" % plan["bot"]["policy"] \
            + BOT_LUA.read_text(encoding="utf-8")
    src = (
        f"-- 生成文件：勿手改。来源 {relp(scenario_path)}"
        f"（由 tools/scenario_run.py 生成）\n"
        f"local PLAN = {to_lua(plan_lua)}\n"
        + bot_src
        + harness
    )
    return src, hashlib.sha256(src.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------- 运行

def scenario_path_of(name_or_path: str) -> Path:
    p = Path(name_or_path)
    if p.is_file():
        return p
    cand = SCENARIO_DIR / f"{name_or_path}.txt"
    if cand.is_file():
        return cand
    raise ScenarioError(f"找不到场景 {name_or_path!r}（试过 {p} 与 {cand}）")


def archive_existing(name: str) -> str | None:
    """产物已存在 ⇒ 按时间戳归档（t24 事故：重跑踩踏他人在用的 trace；归档=run 隔离）。"""
    stamp = time.strftime("%Y%m%d-%H%M%S")
    dst = TRACES_DIR / ".archive" / f"{name}-{stamp}"
    moved = False
    for pat in (f"{name}.frames.jsonl", f"{name}.captures.jsonl", f"{name}.done",
                f"{name}.run.json", f"{name}.lua.log", f"{name}.mesen.log",
                f"{name}.inputs.jsonl", f"{name}.events.jsonl",
                f"{name}.gen.lua", f"{name}.probe.jsonl"):
        src = TRACES_DIR / pat
        if src.exists():
            dst.mkdir(parents=True, exist_ok=True)
            shutil.move(str(src), str(dst / pat))
            moved = True
    for src in (TRACES_DIR).glob(f"{name}_f*.ram.hex"):
        dst.mkdir(parents=True, exist_ok=True)
        shutil.move(str(src), str(dst / src.name))
        moved = True
    for src in (SHOTS_DIR).glob(f"{name}_f*.png"):
        shots_dst = dst / "shots"
        shots_dst.mkdir(parents=True, exist_ok=True)
        shutil.move(str(src), str(shots_dst / src.name))
        moved = True
    return str(dst.relative_to(ROOT)) if moved else None



def historical_first_sample_s(name: str, first_sample_frame: int) -> list[float]:
    """历史 run 的『启动→首采样帧』秒数样本：新字段 first_sample_s 直读；旧 run 以
    wall_s × 首采样帧/总帧 估算（同一驱动/机器，比例模型足够做 3 倍告警基线）。"""
    samples: list[float] = []
    for d in sorted((TRACES_DIR / ".archive").glob(f"{name}-*")):
        mp = d / f"{name}.run.json"
        if not mp.is_file():
            continue
        try:
            m = json.loads(mp.read_text(encoding="utf-8"))
        except Exception:
            continue
        if "first_sample_s" in m and m["first_sample_s"]:
            samples.append(float(m["first_sample_s"]))
        elif m.get("wall_s") and m.get("frames"):
            samples.append(float(m["wall_s"]) * first_sample_frame / max(1, int(m["frames"])))
    return samples


def normalize_paths_in_outputs(name: str) -> None:
    """产物 JSON 禁绝对路径（可移植性）：把 ROOT 前缀归一为仓库相对路径。

    Lua 落盘需要绝对目录（PLAN.shots_dir），但 captures.jsonl/run.json 对外
    暴露的路径必须相对——2026-10-14 用户裁定。

    Windows（headless 驱动）额外注意：Lua 侧路径为 `D:\\workspace\\...`，写进 JSON 后
    反斜杠被转义成 `D:\\\\workspace\\\\...`；且 `/` 与 `\\` 可能混用（`shots/...`）。
    因此对 ROOT 的 4 种字面形式（原生/正斜杠 × 原始/JSON 转义）全部归一，并在归一后
    断言产物中不再出现绝对路径（fail-closed，避免"看起来归一了其实没归"）。
    """
    root_s = str(ROOT)
    variants = {
        root_s + "/",
        root_s + "\\",
        ROOT.as_posix() + "/",
        root_s.replace("\\", "\\\\") + "\\\\",  # JSON 转义后的 `D:\\workspace\\...`
        root_s.replace("\\", "\\\\") + "/",
        ROOT.as_posix().replace("/", "\\/") + "/",
    }
    for f in [TRACES_DIR / f"{name}.captures.jsonl", TRACES_DIR / f"{name}.run.json"]:
        if not f.is_file():
            continue
        txt = f.read_text(encoding="utf-8")
        for p in variants:
            if p in txt:
                txt = txt.replace(p, "")
        f.write_text(txt, encoding="utf-8", newline="\n")
        # 零绝对路径回归断言：捕获 `"C:\\` / `"D:/` 之类仓库绝对路径残留。
        for bad in (f'"{root_s[0]}:\\\\', f'"{root_s[0]}:/'):
            if bad in txt:
                rel = f.relative_to(ROOT).as_posix()
                raise ScenarioError(
                    f"产物仍含绝对路径（归一失败）：{rel} 命中 {bad!r}；"
                    "请检查 normalize_paths_in_outputs 的转义变体"
                )


def done_ok(manifest: dict) -> bool:
    """采集完成判据（task-110：**以 DONE 为准**，plan frames 仅是上限）。

    - `done` 非空且未 `timed_out`；
    - 磁盘已写 trace_lines > 0；
    - DONE 自带 `trace_lines`（task-105 新格式）⇒ 必须等于磁盘行数
      （bot 早停 `trace_lines<plan` / overshoot `trace_lines>plan` 均属合法完成）；
    - legacy DONE 无 `trace_lines` 字段 ⇒ 回退旧判据 `trace_lines == plan.frames`；
    - **`done.frame_errors == 0`**（task-105 pcall 韧性后异常被计数而非静默；legacy 无字段视为 0）
      —— 有帧级异常即视为降级完成，不接受为 PASS。
    """
    done = manifest.get("done")
    if done is None or manifest.get("timed_out", False):
        return False
    n_lines = int(manifest.get("trace_lines", 0) or 0)
    if n_lines <= 0:
        return False
    try:
        if int(done.get("frame_errors", 0) or 0) != 0:
            return False
        if "trace_lines" in done:
            done_lines = int(done["trace_lines"])
            return n_lines == done_lines
    except (TypeError, ValueError):
        return False
    return n_lines == int(manifest.get("frames", -1))


def clear_mesen_resume_state(rom: Path) -> list[str]:
    """清除 Mesen2 的 recent-game resume 状态（跨 run 污染源，task-133/F134-1 第二源）。

    Mesen2 GUI 退出时会把 resume state **内嵌**写进 `RecentGames/<rom>.rgd`（内含
    `Savestate.mss`），下一次启动同一 ROM 可能自动恢复该状态 ⇒ trace 第二跑/跨模式出现不同
    sha（实测 title-menu run1=c4d6…、run2=57e2…；删除 .rgd 后恢复一致）。采集前显式删除，
    不碰 settings.json；返回被删除文件（便于报告留痕）。
    """
    if os.name != "nt":
        return []
    home = Path(os.environ.get("GUNNAC_MESEN_HOME",
                               str(Path(os.environ.get("USERPROFILE", "")) / "Documents" / "Mesen2")))
    removed: list[str] = []
    for pat in (home / "RecentGames" / f"{rom.stem}.rgd",):
        if pat.is_file():
            pat.unlink()
            removed.append(str(pat))
    return removed


def first_lua_error(manifest: dict) -> str | None:
    """取 lua.log 首条逐帧异常行（task-110 F1：FAIL/DEGRADED 时打印 frame_errors 与首条错误）。"""
    trace = (manifest.get("outputs") or {}).get("trace")
    if not trace:
        return None
    log_path = Path(str(trace).replace(".frames.jsonl", ".lua.log"))
    try:
        with open(log_path, encoding="utf-8") as f:
            for line in f:
                if ("endFrame error" in line or "inputPolled error" in line
                        or "wants_stop error" in line):
                    return line.strip()
    except OSError:
        return None
    return None


def run_scenario(name_or_path: str, timeout_s: float = 1800.0, keep_gen: bool = True) -> dict:
    spath = scenario_path_of(name_or_path)
    plan = parse_scenario(spath)
    name = plan["name"]
    # 快速前置检查（F3/F4）：缺 ROM / 缺 Mesen 立即失败，不干等到 timeout。
    if not ROM.is_file():
        raise ScenarioError(f"ROM 不存在：{ROM}（GUNNAC_ROM 可覆盖）")
    if not MESEN.is_file():
        raise ScenarioError(
            f"Mesen2 不存在：{MESEN}（GUNNAC_MESEN 可覆盖；"
            "Windows 需 tools/mesen2/Mesen.exe，POSIX 需 tools/mesen2/Mesen）"
        )
    # 串行锁（t24 事故）：同一时刻只允许一个 Mesen 场景运行。
    # POSIX=flock；Windows（headless 驱动）=msvcrt 锁 1 字节（无 fcntl 依赖）。
    (ROOT / "analysis").mkdir(parents=True, exist_ok=True)
    lock_f = open(ROOT / "analysis" / ".scenario.lock", "a+")
    try:
        if fcntl is not None:
            fcntl.flock(lock_f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        else:
            import msvcrt  # type: ignore
            lock_f.seek(0)
            msvcrt.locking(lock_f.fileno(), msvcrt.LK_NBLCK, 1)
    except OSError:
        lock_f.close()
        raise ScenarioError("另一场景运行中（串行纪律）：analysis/.scenario.lock 被占用")
    # t54 F3 根因防护：串行锁只罩驱动进程——驱动崩溃/被强杀后 Mesen/Xvfb 孤儿仍在跑，
    # 与后续运行争显示/写同一批产物（实测 trace 交错、DONE 不落、30x 变慢）。启动前显式清扫。
    # 仅 POSIX GUI 路径有 Xvfb/pgrep：headless 与 Windows GUI 跳过（Windows 无 pgrep）。
    if (not HEADLESS) and os.name != "nt":
        stale = subprocess.run(["pgrep", "-f", "tools/mesen2/Mesen"], capture_output=True, text=True).stdout.split()
        if stale:
            for pid in stale:
                try:
                    os.kill(int(pid), signal.SIGKILL)
                except (ProcessLookupError, ValueError):
                    pass
            print(f"  [清场] 清除孤儿 Mesen {len(stale)} 个（驱动侧锁覆盖不到的残留）")
    archived = archive_existing(name)
    # F2（复核 task-80）：per-scenario 截图/产物若未被 archive_existing 清走（历史遗留、
    # 手工实验产物），会进入 glob 计数并与本次写入混合 ⇒ 双跑误判。这里再次显式清场：
    # 把残留 `<name>_f*.png` / `<name>_f*.ram.hex` 移入本次归档目录，跑前 shots/traces 干净。
    def _clear_leftovers() -> None:
        leftover_shots = sorted(SHOTS_DIR.glob(f"{name}_f*.png")) if SHOTS_DIR.is_dir() else []
        leftover_rams = sorted(TRACES_DIR.glob(f"{name}_f*.ram.hex")) if TRACES_DIR.is_dir() else []
        if not (leftover_shots or leftover_rams):
            return
        stamp = time.strftime("%Y%m%d-%H%M%S")
        dst = TRACES_DIR / ".archive" / f"{name}-{stamp}-leftovers"
        (dst / "shots").mkdir(parents=True, exist_ok=True)
        for p in leftover_shots:
            shutil.move(str(p), str(dst / "shots" / p.name))
        for p in leftover_rams:
            shutil.move(str(p), str(dst / p.name))
        print(f"  [清场] 残留产物 {len(leftover_shots)} 截图 / {len(leftover_rams)} RAM 转入 {dst.relative_to(ROOT)}")
    _clear_leftovers()
    (TRACES_DIR).mkdir(parents=True, exist_ok=True)
    (SHOTS_DIR).mkdir(parents=True, exist_ok=True)
    GEN_DIR.mkdir(parents=True, exist_ok=True)
    lua_src, lua_sha = generate_lua(plan, spath)
    lua_path = GEN_DIR / f"{name}.lua"
    lua_path.write_text(lua_src, encoding="utf-8", newline="\n")
    # 工具态留档（复核 #82 建议 4——旧生成 lua 被覆盖 ⇒ "旧态不可复原"）：生成 lua 同步落
    # analysis/traces/<name>.gen.lua 持久副本，随 run 产物归档（archive_existing 一并迁移）。
    gen_archive = TRACES_DIR / f"{name}.gen.lua"
    gen_archive.write_text(lua_src, encoding="utf-8", newline="\n")

    # t54：观测/基线设置全部在 Mesen 启动前完成（此段崩溃不得孤儿化模拟器）
    cap_frames = [int(c["frame"]) for c in plan.get("captures", []) if "frame" in c]
    every = plan.get("capture_every")
    if isinstance(every, dict) and every.get("n"):
        cap_frames.append(int(every["n"]))
    first_sample_frame = min(cap_frames) if cap_frames else 60
    history = historical_first_sample_s(name, first_sample_frame)
    median_s = sorted(history)[len(history) // 2] if history else None
    # 地板 15s 吸收 Mesen 启动开销（比例模型不含启动段；真 first_sample_s 样本入史后中位数自校准）
    warn_at = max((3.0 * median_s) if median_s else 120.0, 15.0)
    first_sample_s: float | None = None
    next_progress = 60.0
    probe_globs = [
        TRACES_DIR / f"{name}_f*.ram.hex",
        SHOTS_DIR / f"{name}_f*.png",
    ]

    done_path = TRACES_DIR / f"{name}.done"
    trace_path = TRACES_DIR / f"{name}.frames.jsonl"
    if done_path.exists():
        done_path.unlink()

    t0 = time.time()
    mesen_log = open(TRACES_DIR / f"{name}.mesen.log", "w", encoding="utf-8", newline="\n")
    # 自管 Xvfb（t24 实测：xvfb-run 包装层偶发提前退出 → Xvfb 随之死 → Mesen 冻在中途帧、
    # 场景仍必须**串行**运行（run-all 已是串行；勿并行多个 Mesen）。
    # headless（Mesen2 .exe + --testRunner <lua> <rom>）：不需要显示/Xvfb；脚本 emu.stop 后
    # testRunner 自行退出。GUI 模式（Linux）：自管 Xvfb + DISPLAY 注入（原逻辑保留）。
    xvfb = None
    # Mesen2 testRunner 自带超时（源码 TestRunnerTimeout 默认 100s）——长场景（stg08 34000f 等）
    # 必须在命令行显式给 `--timeout=<秒>`，否则会在 ~100s 处被强停（trace 截断、DONE 不落）。
    mesen_timeout_s = max(120, int(timeout_s) + 120)
    # 跨 run 状态污染防护（task-133/F134-1 第二源）：删除 Mesen2 recent-game resume state。
    removed_states = clear_mesen_resume_state(ROM)
    if removed_states:
        print(f"  [清场] 删除 Mesen2 resume 状态: {', '.join(Path(p).name for p in removed_states)}")
    try:
        if HEADLESS:
            proc = subprocess.Popen(
                [str(MESEN), "--testRunner", f"--timeout={mesen_timeout_s}", str(lua_path), str(ROM),
                 "/DoNotSaveSettings"],
                cwd=ROOT, stdout=mesen_log, stderr=subprocess.STDOUT, text=True,
            )
        elif GUI_WINDOWS:
            # Windows GUI 模式：窗口（真实输入注入可用）；脚本位置参数 + AutoStartScriptOnLoad。
            proc = subprocess.Popen(
                [str(MESEN), str(ROM), str(lua_path), "/DoNotSaveSettings"],
                cwd=ROOT, stdout=mesen_log, stderr=subprocess.STDOUT, text=True,
            )
        else:
            display = pick_display()
            xvfb = subprocess.Popen(
                ["Xvfb", display, "-screen", "0", "1280x1024x24", "-nolisten", "tcp"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True,
            )
            wait_for_x(display, xvfb)
            env = dict(os.environ)
            env["DISPLAY"] = display
            proc = subprocess.Popen(
                [str(MESEN), str(ROM), str(lua_path), "/DoNotSaveSettings"],
                cwd=ROOT, stdout=mesen_log, stderr=subprocess.STDOUT, text=True,
                start_new_session=True, env=env,
            )
    except FileNotFoundError as exc:
        mesen_log.close()
        if xvfb is not None:
            xvfb.kill()
        raise ScenarioError(f"启动 Mesen2 失败（可执行/依赖缺失）：{exc}")

    timed_out = False
    dead_since: float | None = None
    try:
        while not done_path.exists():
            now = time.time()
            el = now - t0
            if now - t0 > timeout_s:
                timed_out = True
                break
            if proc.poll() is not None:
                # 包装层退出 ≠ 采集完成（孤儿 Mesen 可能仍在写盘）；给 20s 宽限等 DONE
                if dead_since is None:
                    dead_since = now
                elif now - dead_since > 20:
                    break
            if first_sample_s is None:
                if any(any(g.parent.glob(g.name)) for g in probe_globs):
                    first_sample_s = round(el, 1)
                    print(f"  首采样帧（~f{first_sample_frame}）耗时 {first_sample_s}s"
                          f"（历史中位 {('%.1f' % median_s) if median_s else '无'}s）")
                elif el > next_progress:
                    print(f"  [时长告警] 已 {el:.0f}s 未见首采样产物（基线 {warn_at:.0f}s）"
                          "——疑似挂起/负载异常，检查 Mesen/Xvfb/系统负载", flush=True)
                    next_progress = el + 60.0
            time.sleep(0.5)
    finally:
        def kill_group(sig) -> None:
            if os.name == "nt":
                try:
                    proc.kill()
                except OSError:
                    pass
                return
            try:
                os.killpg(os.getpgid(proc.pid), sig)
            except (ProcessLookupError, PermissionError, OSError):
                pass

        if proc.poll() is None or not done_path.exists():
            kill_group(signal.SIGTERM)
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            kill_group(signal.SIGKILL)
            proc.wait(timeout=10)
        mesen_log.close()
        if xvfb is not None:
            try:
                xvfb.terminate()
                xvfb.wait(timeout=5)
            except Exception:
                try:
                    xvfb.kill()
                except Exception:
                    pass
    wall = round(time.time() - t0, 1)

    done_info = None
    if done_path.exists():
        done_info = json.loads(done_path.read_text(encoding="utf-8").strip().splitlines()[-1])
    n_lines = 0
    if trace_path.exists():
        with open(trace_path, encoding="utf-8") as f:
            n_lines = sum(1 for _ in f)
    shots = sorted(p.relative_to(ROOT).as_posix() for p in (SHOTS_DIR).glob(f"{name}_f*.png"))
    rams = sorted(p.relative_to(ROOT).as_posix() for p in (TRACES_DIR).glob(f"{name}_f*.ram.hex"))

    manifest = {
        "scenario": relp(spath),
        "scenario_sha256": hashlib.sha256(spath.read_bytes()).hexdigest(),
        "generated_lua": lua_path.relative_to(ROOT).as_posix() if keep_gen else None,
        "generated_lua_sha256": lua_sha,
        "generated_lua_archive": gen_archive.relative_to(ROOT).as_posix(),
        "frames": plan["frames"],
        "trace_lines": n_lines,
        "done": done_info,
        "timed_out": timed_out,
        "wall_s": wall,
        "first_sample_s": first_sample_s,
        "duration_warning": (
            f"首采样 {first_sample_s}s 超历史中位 {median_s:.1f}s 的 3 倍——疑似挂起/负载异常"
            if first_sample_s is not None and median_s is not None and first_sample_s > 3.0 * median_s
            else None
        ),
        "outputs": {
            "trace": trace_path.relative_to(ROOT).as_posix(),
            "captures": f"analysis/traces/{name}.captures.jsonl",
            "shots": shots,
            "ram_dumps": rams,
            **({"bot_inputs": f"analysis/traces/{name}.inputs.jsonl",
                "bot_events": f"analysis/traces/{name}.events.jsonl"} if plan.get("bot") else {}),
        },
    }
    mpath = TRACES_DIR / f"{name}.run.json"
    mpath.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")

    if fcntl is not None:
        fcntl.flock(lock_f, fcntl.LOCK_UN)
    else:
        try:
            import msvcrt  # type: ignore
            lock_f.seek(0)
            msvcrt.locking(lock_f.fileno(), msvcrt.LK_UNLCK, 1)
        except OSError:
            pass
    lock_f.close()
    if archived:
        print(f"  已归档旧产物: {archived}（run 隔离，防踩踏）")
    ok = done_ok(manifest)
    print(f"== scenario run: {name} ==")
    print(f"  DSL: {manifest['scenario']} sha256={manifest['scenario_sha256'][:16]}…")
    normalize_paths_in_outputs(name)
    print(f"  frames: {plan['frames']} trace_lines: {n_lines} captures: {len(rams)} shots: {len(shots)} wall: {wall}s")
    if done_info:
        print(f"  DONE: {json.dumps(done_info, ensure_ascii=False)}")
    print(f"  trace: {manifest['outputs']['trace']}")
    print(f"  run清单: {mpath.relative_to(ROOT)}")
    if not ok:
        print(f"  !! 未完成（timed_out={timed_out}, done={done_info is not None}, trace_lines={n_lines}）")
    return manifest


def cmd_list(_args) -> int:
    rows = []
    for p in sorted(SCENARIO_DIR.glob("*.txt")):
        try:
            plan = parse_scenario(p)
            rows.append((plan["name"], plan["frames"], len(plan["presses"]), len(plan["holds"]), len(plan["captures"])))
        except ScenarioError as exc:
            rows.append((p.name, "解析失败", str(exc), "", ""))
    print(f"{'场景':<24} {'帧数':>6} {'press':>6} {'hold':>5} {'capture':>8}")
    for r in rows:
        print(f"{r[0]:<24} {str(r[1]):>6} {str(r[2]):>6} {str(r[3]):>5} {str(r[4]):>8}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(prog="tools/scenario_run.py", description="S2 oracle 场景采集驱动（DSL 唯一解析器）")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list", help="列出 oracle/scenarios/ 场景")
    p_gen = sub.add_parser("gen", help="只生成 lua（调试）")
    p_gen.add_argument("scenario")
    p_run = sub.add_parser("run", help="运行单个场景")
    p_run.add_argument("scenario")
    p_run.add_argument("--timeout", type=float, default=1800.0)
    p_all = sub.add_parser("run-all", help="顺序运行全部场景")
    p_all.add_argument("--timeout", type=float, default=1800.0)
    args = ap.parse_args()

    try:
        if args.cmd == "list":
            return cmd_list(args)
        if args.cmd == "gen":
            spath = scenario_path_of(args.scenario)
            plan = parse_scenario(spath)
            GEN_DIR.mkdir(parents=True, exist_ok=True)
            src, sha = generate_lua(plan, spath)
            out = GEN_DIR / f"{plan['name']}.lua"
            out.write_text(src, encoding="utf-8", newline="\n")
            print(f"生成 {out.relative_to(ROOT)} sha256={sha}")
            return 0
        if args.cmd == "run":
            m = run_scenario(args.scenario, args.timeout)
            # F90-1（task-90 复核）：run 分支不再固定 return 0——未完成（done=None / timed_out /
            # 截断 trace）必须非零退出，供 P0-0 自动化与 CI 判失败（不能只看 timed_out：
            # Mesen2 testRunner 100s 自停时 timed_out 仍为 false）。
            # task-110：判据以 DONE 为准（bot 早停/overshoot 合法 ⇒ 不因 trace_lines!=plan 误判）。
            if not done_ok(m):
                done = m.get("done") or {}
                err = first_lua_error(m)
                print(
                    "RUN: FAIL（done={} timed_out={} trace_lines={}/frames={} done_trace_lines={} frame_errors={}）".format(
                        m.get("done") is not None,
                        m.get("timed_out"),
                        m.get("trace_lines"),
                        m.get("frames"),
                        done.get("trace_lines"),
                        int(done.get("frame_errors", 0) or 0),
                    )
                )
                if err:
                    print(f"  首条帧级异常：{err}")
                return 1
            return 0
        if args.cmd == "run-all":
            failed = []
            degraded = []
            for p in sorted(SCENARIO_DIR.glob("*.txt")):
                m = run_scenario(p, args.timeout)
                if not done_ok(m):
                    failed.append(m["scenario"])
                    done = m.get("done") or {}
                    fe = int(done.get("frame_errors", 0) or 0)
                    if done and fe > 0:
                        err = first_lua_error(m)
                        degraded.append((m["scenario"], fe, err))
            if degraded:
                for nm, fe, err in degraded:
                    print(f"RUN-ALL: DEGRADED {nm} frame_errors={fe}" + (f" 首条={err}" if err else ""))
            if failed:
                print(f"RUN-ALL: 未完成 {failed}")
                return 1
            print("RUN-ALL: 全部场景完成")
            return 0
    except ScenarioError as exc:
        print(f"场景错误: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
