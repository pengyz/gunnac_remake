#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/diff_subroutines.py —— L2 子程序级差分门禁驱动（subroutine-diff 门禁入口）。

薄封装：差分逻辑唯一存在于 crates/game-oracle（6502 核 + 三方裁定），本脚本只负责
调 `cargo run -p game-oracle --bin l2-diff` 并透传输出/退出码（无第二套扫描逻辑）。

裁定面（oracle/subroutines/l2_cases.jsonl，先写用例再实现）：
    case golden == 6502 核执行 ROM 原字节出参 == game-core 同名函数出参
用法：
    python3 tools/diff_subroutines.py                # 跑默认 case 集
    python3 tools/diff_subroutines.py --cases <路径>  # 指定 case 文件
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    cmd = [
        "cargo", "run", "-q", "-p", "game-oracle", "--bin", "l2-diff", "--",
        "--rom", str(ROOT / "rom" / "gunnac_cn.nes"),
    ]
    if "--cases" in sys.argv:
        idx = sys.argv.index("--cases")
        if idx + 1 >= len(sys.argv):
            print("用法: diff_subroutines.py [--cases <路径>]", file=sys.stderr)
            return 2
        cmd += ["--cases", sys.argv[idx + 1]]
    else:
        cmd += ["--cases", str(ROOT / "oracle" / "subroutines" / "l2_cases.jsonl")]

    proc = subprocess.run(cmd, cwd=ROOT)
    return proc.returncode


if __name__ == "__main__":
    sys.exit(main())
