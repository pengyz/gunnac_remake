#!/usr/bin/env python3
"""wave_timeline.py — B1 敌群/弹幕时间线取证复现工具（docs/findings/enemy-timeline.md）。

用法：
  python3 tools/wave_timeline.py dir              # 关卡目录（bank8:$A000，8B/条）
  python3 tools/wave_timeline.py script [N]       # 各关脚本流首部命令解码（N=关号 1..8）
  python3 tools/wave_timeline.py sched [N]        # 各关敌波调度流（sub_86AE 流）解码
  python3 tools/wave_timeline.py spawns [run]     # trace 网格 slot 激活事件（默认 gameplay-stage1）

截断口径（复核 #76 F1/F5）：script 解码至流尾（op>11/跳转收尾）；触及 maxrec 安全上限
一律显式打 [截断] 提示，禁止以截断视界下范围结论。sched 的 +N 为 tick 累计值（1 tick=8 帧稳态）。

证据口径：bank00:8000 装载器 / 8185 脚本解释器 / 82FD 事件机 / 83E7 生成器 /
844A 行为 VM / 86AE 敌波调度器 / 8972 波次记录机（详见 enemy-timeline.md §1）。
"""
import json
import sys
import os

ROM = "rom/gunnac_cn.nes"
TRACE = "analysis/traces/{run}.frames.jsonl"

# 脚本流命令字节码 → 操作数长度（bank00:81D0 分派表，逐 handler 清点）
SCRIPT_OPS = {0: 3, 1: 4, 2: 8, 3: 2, 4: 3, 5: 2, 6: 2, 7: 2, 8: 2, 9: 2, 10: 2, 11: 1}
SCRIPT_NAME = {
    0: "setspeed($80/$81/$7F)",
    1: "set-gen(evt=$97/$98, spawner=$A1/$A2)",
    2: "set-spawnsched($4F/$50,$42/$43,$3E/$3F,$40/$41)",
    3: "jump",
    4: "set-tbl($181/$182,ptr)",
    5: "set-wavestream($CC/$CD)",
    6: "set-spawner($A1/$A2)",
    7: "jump-if-wave-busy",
    8: "set-8($63/$64)",
    9: "set-9($63/$64)",
    10: "set-10($75/$76)",
    11: "nop",
}


def fb(bank, addr):
    """bank 内规范 CPU 地址（$A000 窗）→ 文件偏移（bank 数据在 R7 窗）"""
    return 0x10 + bank * 0x2000 + (addr - 0xA000)


def load():
    return open(ROM, "rb").read()


def dump_dir(d):
    print("== 关卡目录 bank8:$A000（bank00:8000 8B/条；Y=$180*8）==")
    print("idx  $180  script  f2/3    b7d  bank  $9F(LUT)")
    for i in range(9):
        e = d[0x10010 + i * 8:0x10010 + i * 8 + 8]
        print(
            f"{i:3d}  {i:4d}  ${e[0] | e[1] << 8:04X}  ${e[2] | e[3] << 8:04X}  "
            f"{e[4]:02X}   {e[5]:02X}   ${e[6] | e[7] << 8:04X}"
        )


def stage_entry(d, n):
    e = d[0x10010 + n * 8:0x10010 + n * 8 + 8]
    return dict(script=e[0] | e[1] << 8, f23=e[2] | e[3] << 8, b7d=e[4],
                bank=e[5], lut=e[6] | e[7] << 8)


def decode_script(d, n, maxrec=256):
    """脚本流解码到流尾（op>11/跳转收尾）。maxrec 仅为安全上限——触及时显式打截断提示
    （复核 #76 F1 根因：旧 maxrec=8 静默截断曾漏计 op5 波次流）。"""
    st = stage_entry(d, n)
    off = fb(st["bank"], st["script"])
    print(f"== STG-{n:02d} 脚本流 ${st['script']:04X} (bank{st['bank']}) "
          f"目录: evt-gen/spawner 见 op1, 调度流见 op2 ==")
    t0 = d[off] | d[off + 1] << 8
    print(f"  头 2B 初始 tick T0=${t0:04X}（$84/$85:=T0-1, $86/$87:=T0）")
    p = off + 2
    rec = 0
    waves = []   # (记录序, 波次流指针, 记录尾 thr tick)
    ended = False
    for _ in range(maxrec):
        op = d[p]
        if op > 11:
            print(f"  @{p - off:+3d}: opcode ${op:02X} > 11 —— 停")
            ended = True
            break
        ln = SCRIPT_OPS[op]
        args = d[p + 1:p + 1 + ln]
        thr = d[p + 1 + ln] | d[p + 2 + ln] << 8
        rec += 1
        extra = ""
        if op == 1:
            extra = f" evt=${args[0] | args[1] << 8:04X} spawner=${args[2] | args[3] << 8:04X}"
        elif op == 6:
            extra = f" spawner=${args[0] | args[1] << 8:04X}"
        elif op == 2:
            extra = (f" sched=${args[0] | args[1] << 8:04X}"
                     f" s2=${args[2] | args[3] << 8:04X}"
                     f" s3=${args[4] | args[5] << 8:04X}"
                     f" s4=${args[6] | args[7] << 8:04X}")
        elif op == 5:
            extra = f" wave=${args[0] | args[1] << 8:04X}"
            waves.append((rec, args[0] | args[1] << 8, thr))
        elif op in (3, 7):
            extra = f" target=${args[0] | args[1] << 8:04X}"
        print(f"  @{p - off:+3d} rec{rec:<3d}: op{op:<2d} {SCRIPT_NAME[op]:42s} "
              f"args={args.hex(' ')} thr=${thr:04X}{extra}")
        p += 1 + ln + 2
        if op in (3, 7):
            ended = True
            break
    if not ended:
        print(f"  [截断] 已达 maxrec={maxrec} 且流未以 op>11/跳转收尾——以上为不完整视界，"
              f"勿据此下范围结论（复核 #76 F1 根因）")
    print(f"  op5 波次流 {len(waves)} 条: "
          + ("、".join(f"${w:04X}@thr${t:04X}(rec{r})" for r, w, t in waves) if waves else "无")
          + "（@thr=记录尾 thr 字段，即 $86/$87 下一记录触发 tick）")


def sched_stream_ptr(d, n):
    """STG-N 脚本流首个 op2 记录的调度流指针（$4F/$50）；找不到返回 None。"""
    st = stage_entry(d, n)
    off = fb(st["bank"], st["script"])
    p = off + 2
    for _ in range(12):
        op = d[p]
        if op > 11:
            break
        ln = SCRIPT_OPS[op]
        if op == 2:
            return d[p + 1] | d[p + 2] << 8
        p += 1 + ln + 2
    return None


def iter_sched_records(d, n, maxrec=40):
    """敌波调度流记录迭代器（bank00:86CE 记录机字节语义，与 decode_sched 同源唯一实现）。

    产出 dict：{'addr','tick','items','next_tick','jump','truncated','loop'}
      items: ('pair', type, param) | ('ctrl', ctrl, type, args4) | ('jump', ctrl, target)
             | ('set', ctrl, a, b) | ('unk', ctrl)
      tick = 该记录触发的累计 delay tick（+N 口径，见 decode_sched 注）。
    复杂度/口径同 decode_sched（复核 #76 §8.1：sched maxrec=40 默认；B2 传 maxrec≥200）。
    """
    st = stage_entry(d, n)
    sched = sched_stream_ptr(d, n)
    if sched is None:
        return
    base = fb(st["bank"], sched)
    ticks = d[base] + 1
    rb = 0
    seen = set()
    recno = 0
    while recno < maxrec:
        key = (st["bank"], sched + rb)
        if key in seen:
            yield {"addr": sched + rb, "tick": ticks, "items": [], "loop": True,
                   "truncated": False}
            return
        seen.add(key)
        y = rb + 1
        items = []
        nxt = None
        jump = None
        while True:
            v = d[base + y]
            if v >= 0x10:
                items.append(("pair", v, d[base + y + 1]))
                y += 2
            else:
                ctrl = v
                y += 1
                if ctrl == 0:
                    nxt = y
                    break
                if ctrl in (1, 11, 2, 12):
                    t = {1: 0x2E, 11: 0x2C, 2: 0x2F, 12: 0x2D}[ctrl]
                    a = tuple(d[base + y:base + y + 4])
                    items.append(("ctrl", ctrl, t, a))
                    y += 4
                elif 3 <= ctrl <= 9:
                    tgt = d[base + y] | d[base + y + 1] << 8
                    items.append(("jump", ctrl, tgt))
                    jump = tgt
                    break
                elif ctrl == 10:
                    items.append(("set", 10, d[base + y], d[base + y + 1]))
                    y += 2
                elif ctrl in (13, 14):
                    items.append(("set", ctrl, d[base + y], d[base + y + 1]))
                    y += 2
                else:
                    items.append(("unk", ctrl))
                    nxt = y
                    break
        yield {"addr": sched + rb, "tick": ticks, "items": items, "loop": False,
               "truncated": False}
        recno += 1
        if jump is not None:
            off2 = fb(st["bank"], jump)
            ticks += d[off2]
            sched = jump
            base = off2
            rb = 0
        elif nxt is not None:
            ticks += d[base + nxt]
            rb = nxt
        else:
            return
    yield {"addr": None, "tick": ticks, "items": [], "loop": False, "truncated": True}


def decode_sched(d, n, maxrec=40):
    """sub_86AE 敌波调度流解码（bank00:86CE 记录机）。
    记录 = [delay@0][项…][ctrl]；项 = (type≥$10, param) 对 或 ctrl：
      0=记录分隔 1/11=定型$2E/$2C+4参 2/12=定型$2F/$2D+($3E)参+4参
      3..9=流跳转(2B) 10=流定型+2参 13/14=寄存器写(2参)
    单位：+N 为 delay **tick** 累计值（1 tick = 8 帧稳态；关首快进环另计——复核 #76 §1）。
    maxrec 截断时显式打提示（复核 #76 F5）。字节语义唯一实现 = iter_sched_records（本函数只打印）。
    """
    sched = sched_stream_ptr(d, n)
    if sched is None:
        print("未找到 op2 调度流指针")
        return
    st = stage_entry(d, n)
    base = fb(st["bank"], sched)
    print(f"== STG-{n:02d} 敌波调度流 ${sched:04X} (bank{st['bank']}, sub_86AE) ==")
    print(f"  初始 gap = {d[base] + 1}t（tick：$51:=(@0)+1；1 tick=8 帧稳态）")
    for r in iter_sched_records(d, n, maxrec=maxrec):
        if r["truncated"]:
            print(f"  [截断] 已达 maxrec={maxrec}——以上为不完整视界，勿据此下范围结论"
                  f"（复核 #76 F5：截断须显式提示）")
            break
        if r["loop"]:
            print(f"  [循环回 ${r['addr']:04X}] —— 停")
            break
        rec = []
        for it in r["items"]:
            if it[0] == "pair":
                rec.append(f"(${it[1]:02X},${it[2]:02X})")
            elif it[0] == "ctrl":
                ctrl, t, a = it[1], it[2], it[3]
                rec.append(f"[{ctrl}]T=${t:02X}({a[0]:02X},{a[1]:02X},{a[2]:02X},{a[3]:02X})")
            elif it[0] == "jump":
                rec.append(f"[{it[1]}]JMP ${it[2]:04X}")
            elif it[0] == "set":
                rec.append(f"[{it[1]}]reg({it[2]:02X},{it[3]:02X})" if it[1] in (13, 14)
                           else f"[10]T=${it[2]:02X}(p=${it[3]:02X})")
            else:
                rec.append(f"[{it[1]}]?")
        print(f"  @{r['addr']:04X} +{r['tick']:6d}t : {' '.join(rec)}")


def trace_spawns(run="gameplay-stage1"):
    path = TRACE.format(run=run)
    print(f"== trace 网格 slot 激活事件（{path}）==")
    print("判定式：iram $0400+n 由 0→非0（字段基址+槽号，enemy-dataface §2.1 正寻址）")
    print("口径注：槽复用写入（旧值非 0→新值）不可见——计数为判定式口径，非出敌总数"
          "（复核 #76 A1：B2 输入生成按此口径会漏槽复用事件）")
    prev = [None] * 28
    cnt = 0
    with open(path) as f:
        for line in f:
            d = json.loads(line)
            ir = bytes.fromhex(d["w"]["iram"])
            for n in range(28):
                t = ir[0x100 + n]
                if prev[n] is not None and prev[n] == 0 and t != 0:
                    x, y = ir[0x1E0 + n], ir[0x1FC + n]
                    blo, bhi = ir[0x138 + n], ir[0x11C + n]
                    f4, f5 = ir[0x170 + n], ir[0x18C + n]
                    print(f"f{d['f']:<5d} slot{n:<2d} type=${t:02X} x={x:3d} y={y:3d} "
                          f"beh=${bhi:02X}{blo:02X} $470=${f4:02X} $48C=${f5:02X}")
                    cnt += 1
                prev[n] = t
    print(f"合计 {cnt} 个激活事件")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return
    d = load()
    cmd = sys.argv[1]
    if cmd == "dir":
        dump_dir(d)
    elif cmd == "script":
        if len(sys.argv) > 2:
            decode_script(d, int(sys.argv[2]))
        else:
            for n in range(1, 9):
                decode_script(d, n)
                print()
    elif cmd == "sched":
        if len(sys.argv) > 2:
            decode_sched(d, int(sys.argv[2]))
        else:
            for n in range(1, 9):
                decode_sched(d, n)
                print()
    elif cmd == "spawns":
        trace_spawns(sys.argv[2] if len(sys.argv) > 2 else "gameplay-stage1")
    else:
        print(__doc__)


if __name__ == "__main__":
    main()
