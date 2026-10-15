#!/usr/bin/env python3
"""帧锚表提取器（stg02-08 §4 填表源；stg02-08 帧锚采集令）。

用法：
  python3 tools/frame_anchor_extract.py <stage 2..8> <trace.jsonl> [--t0 3400]

产出（stdout markdown 片段）：
  §A 记录处理帧表（调度流指针变迁 + $84=+Nt−1 校验 + tick）
  §B op5 波次装载（$CC/$CD 变迁）
  §C 出敌事件表（0→非0 判定式；§2.3 格式）
  §D 停滚检测（$8B/$8C 冻结/归零事件 + tick 冻结值）

口径：事件=iram $0400+槽 0→非0（B1 §2.3；槽复用盲区同源注）。
"""
import json, sys, argparse
import tools.wave_timeline as w


def load_trace(path):
    rows = []
    for line in open(path):
        try:
            rows.append(json.loads(line))
        except Exception:
            pass
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('stage', type=int)
    ap.add_argument('trace')
    ap.add_argument('--t0', type=int, default=None, help='事件表起始帧（默认=关起始推断）')
    a = ap.parse_args()
    rows = load_trace(a.trace)
    print(f'<!-- trace={a.trace} rows={len(rows)} f{rows[0]["f"]}..{rows[-1]["f"]} -->')

    recs = {}
    for r in w.iter_sched_records(w.load(), a.stage, maxrec=300):
        if r.get('loop'):
            break
        recs[r['addr']] = r

    prev_ptr = None
    prev_t = [None] * 28
    prev_cc = None
    trig, spawns, cc = [], [], []
    freeze = []          # ($8B,$8C) 状态变迁
    prev_8b = None
    freeze_frame = None
    t0 = a.t0
    for d in rows:
        f = d['f']
        z = bytes.fromhex(d['w']['zp'])
        ir = bytes.fromhex(d['w']['iram'])
        ptr = (z[0x50] << 8) | z[0x4F]
        tick = (z[0x85] << 8) | z[0x84]
        cur_cc = (z[0xCD] << 8) | z[0xCC]
        pair8b = (z[0x8B], z[0x8C])
        if pair8b != prev_8b:
            freeze.append((f, pair8b))
            prev_8b = pair8b
        if prev_cc is not None and cur_cc != prev_cc:
            cc.append((f, tick, prev_cc, cur_cc))
        prev_cc = cur_cc
        old = prev_ptr
        if old is not None and ptr != old:
            trig.append((f, tick, old, ptr, recs.get(old)))
        if t0 is None and ptr in recs and prev_ptr == 0x0000:
            t0 = f
        if t0 is not None and f >= t0 and old is not None:
            for n in range(28):
                t = ir[0x100 + n]
                if prev_t[n] is not None and prev_t[n] == 0 and t != 0:
                    spawns.append((f, n, t, ir[0x1E0 + n], ir[0x1FC + n],
                                   (ir[0x11C + n] << 8) | ir[0x138 + n],
                                   ir[0x170 + n], ir[0x18C + n], old))
        for n in range(28):
            prev_t[n] = ir[0x100 + n]
        prev_ptr = ptr

    print(f'\n## A 记录处理帧表（t0={t0}；triggers={len(trig)}）')
    print('| 记录 | 静态 +tick | 处理帧 | trace $84 | 校验 |')
    print('|---|---|---|---|---|')
    ok = 0
    for f, tick, old, new, r in trig:
        st = f'+{r["tick"]}t' if r else '—'
        chk = '✓' if (r and tick == r['tick'] - 1) else ('—' if not r else '✗')
        ok += chk == '✓'
        print(f'| @{old:04X}→{new:04X} | {st} | f{f} | ${tick:04X} | {chk} |')
    print(f'\n校验 {ok}/{sum(1 for *_, r in trig if r)}')

    print(f'\n## B op5 波次装载（{len(cc)}）')
    for f, tick, x, y in cc:
        print(f'| f{f} | ${tick:04X} | ${x:04X}→${y:04X} |')

    print(f'\n## C 出敌事件表（{len(spawns)} 事件，f{t0}+）')
    print('| 帧 | 槽 | type | x,y | beh(41C:438) | $470/$48C | 归属 |')
    print('|---|---|---|---|---|---|---|')
    for f, n, t, x, y, beh, p470, p48c, ptr in spawns:
        print(f'| f{f} | {n} | ${t:02X} | ({x},{y}) | ${beh:04X} | ${p470:02X}/${p48c:02X} | @{ptr:04X} |')

    print(f'\n## D 滚动计数变迁（{len(freeze)}；冻结/归零事件）')
    print('| 帧 | $8B/$8C |')
    print('|---|---|')
    for f, (lo, hi) in freeze[-40:]:
        print(f'| f{f} | ${lo:02X}/${hi:02X} |')


if __name__ == '__main__':
    main()
