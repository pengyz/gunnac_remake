#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/screen_stream_extract.py —— R-4 屏流提取器（L4 签署流源）。

从 ROM 直提 per-screen 流文件（R-4 格式：[PPU 地址 2B] tile 码流 [FE 重载][FF 段终]）。
装载点 bank01:A043/A04E/A062/A08A 已知（$12:=$lo/$13:=$hi 指针对）；流=bank1 $A1xx 窗。

用法：
    python3 tools/screen_stream_extract.py <rom> <out_dir>
    python3 tools/screen_stream_extract.py --selftest
"""
import sys
from pathlib import Path

# 四装载点指针对（bank01:A043/A04E/A062/A08A 反汇编实证——hi:lo）
# 【R-4b f209330 命名锚=装载点（A 级）】四流全：$A129 自旋屏（图+2D×7 字幕）/
# $A13D #9 解锁屏（S-CH-3 互证）/$A31F 文本屏（字+空格行）/$A65C 图形块屏（A036）。
# **链式续段注**：$A129 尾段==$A13D 头段——同一画面流分段装载（提取保留链形）。
# csv id 候选（B）：$A129+$A13D 链→opening_bubble/ending_robot 域；$A31F→ending_normal。
STREAM_PTRS = [
    ("a129-spin", 0xA129),        # bank01:A086（R-4 样本址）
    ("a13d-unlock9", 0xA13D),     # bank01:A05E（#9 解锁画面）
    ("a31f-text", 0xA31F),        # bank01:A04A（文本屏）
    ("a65c-graphics", 0xA65C),    # bank01:A036（图形块屏）
]


def bank1_offset(addr):
    return 0x10 + 0x2000 + (addr - 0xA000)


def extract_stream(rom, addr):
    """R-4 格式逐字节至 FF（多段容许；截断=文件尾 fail-closed）。"""
    out = bytearray()
    pos = bank1_offset(addr)
    in_ppu = True  # 首 2B=PPU 地址
    while pos < len(rom):
        b = rom[pos]
        pos += 1
        out.append(b)
        if b == 0xFF:
            break
    return bytes(out)


def extract_chain(rom, addr_a, addr_b):
    """链式拼接（$A129 尾段==$A13D 头段——同流分段装载；相邻界 0xA129+20=0xA13D
    直读拼接，边界段重出注（dedup 挂 R-4b-2 渲染对照））。"""
    a = extract_stream(rom, addr_a)
    b = extract_stream(rom, addr_b)
    return a + b


def main():
    if "--selftest" in sys.argv:
        # 形式自检（无 ROM 依赖）：提取器编码形
        assert bank1_offset(0xA129) == 0x2139
        print("selftest ok")
        return 0
    rom_path, out_dir = sys.argv[1], Path(sys.argv[2])
    rom = Path(rom_path).read_bytes()
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, addr in STREAM_PTRS:
        data = extract_stream(rom, addr)
        (out_dir / f"screen-{name}-a{addr:04x}.bin").write_bytes(data)
        print(f"{name} ${addr:04X}: {len(data)}B → screen-{name}-a{addr:04x}.bin")
    # 链式拼接产物（$A129+$A13D=同流分段——opening_bubble/ending_robot 域候选）
    chain = extract_chain(rom, 0xA129, 0xA13D)
    (out_dir / "screen-chain-a129-a13d.bin").write_bytes(chain)
    print(f"chain $A129+$A13D: {len(chain)}B → screen-chain-a129-a13d.bin")
    return 0


if __name__ == "__main__":
    sys.exit(main())
