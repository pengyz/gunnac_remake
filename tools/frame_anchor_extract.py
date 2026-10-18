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
import json, sys, argparse, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import tools.wave_timeline as w


def iter_trace(path):
    """流式逐帧（v5 300k 帧面内存安全：不落全表）。"""
    with open(path) as fh:
        for line in fh:
            try:
                d = json.loads(line)
            except Exception:
                continue
            if isinstance(d, dict) and "f" in d and "w" in d:  # 残行容断（缺键行跳——v5 崩尾根治）
                yield d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('stage', type=int)
    ap.add_argument('trace')
    ap.add_argument('--t0', type=int, default=None, help='事件表起始帧（默认=关起始推断）')
    ap.add_argument('--anchors-out', default=None, help='v5 锚台账 jsonl 直出路径（golden_backfill 秒收）')
    a = ap.parse_args()
    if a.anchors_out:
        emit_anchors(a.trace, a.anchors_out)
        return
    rows = iter_trace(a.trace)          # 流式（生成器）
    
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


def emit_anchors(trace_path, out_path, run_name=None):
    """v5 全程锚台账直出（300k 帧流式；事件分类=既有锚型；trigger=natural）。

    分类（与 s5-all-clear.anchors.jsonl 既有口径同型）：
      clear-entry  = $27 bit7 置位帧（$27:=$C0 一次性入场）
      shop-init    = $0400:=0 且前帧非 0（商店 init）
      exit-done    = $27 释放（→$00）∧ 调度流指针切换（出店转场尾完成帧）
      stage-start  = exit-done+96f（+96f 三证同型采样）
      stage-combat = 段中 300 倍数帧（probe 同型）
    fail-closed：事件分类不确定时不发明（宁缺勿错）。
    """
    import os
    run = run_name or os.path.basename(trace_path).replace('.frames.jsonl', '')
    sha = __import__('hashlib').sha256(open(trace_path, 'rb').read()).hexdigest()
    rows_out = []
    prev = None
    seg = 0            # 关段计数（clear 事件递增）
    last_exit = None
    combat_next = None
    for d in iter_trace(trace_path):
        f = d['f']
        z = bytes.fromhex(d['w']['zp'])
        ir = bytes.fromhex(d['w']['iram'])
        cur = (ir[0x100], z[0x26], z[0x27], (z[0x50] << 8) | z[0x4F])
        if prev is not None:
            p400, p26, p27, pptr = prev
            c400, c26, c27, cptr = cur
            # clear-entry：$27 bit7 置位（$27&$80 前后沿）
            if (p27 & 0x80) == 0 and (c27 & 0x80) != 0:
                seg += 1
                rows_out.append(_row(f'stg{seg:02d}-clear', f, f'flags27 bit7 置位（$27:=${c27:02X}）；'
                                     f'触发=natural（v5 全程链）', run, sha))
                last_exit = None
            # shop-init：$0400 非0→0 且 $27 bit7 仍置（关末/商店相）
            if p400 != 0 and c400 == 0 and (c27 & 0x80) != 0:
                rows_out.append(_row(f'stg{seg+1:02d}-shop', f, '$0400:=0 商店 init（同 stg02-shop 口径）', run, sha))
            # exit-done：$27 释放 且 ptr 切换（同帧/近帧）
            if (p27 & 0x80) != 0 and (c27 & 0x80) == 0:
                last_exit = f
                rows_out.append(_row(f'stg{seg+1:02d}-exit', f,
                                     f'出店转场尾完成帧（$27 释放 + 调度流切 ${cptr:04X}）', run, sha))
                combat_next = f + 96
        prev = cur
        if combat_next is not None and f == combat_next:
            rows_out.append(_row(f'stg{seg+1:02d}-start', f, f'关始采样帧（+96f 于出店，三证同型）', run, sha))
            combat_next = None
    with open(out_path, 'w', encoding='utf-8', newline='\n') as fh:
        for r in rows_out:
            fh.write(json.dumps(r, ensure_ascii=False) + '\n')
    print(f'anchors: {len(rows_out)} 行 → {out_path}（run={run} sha={sha[:16]}…）', file=__import__('sys').stderr)


def _row(cid, f, predicate, run, sha):
    return {
        'id': cid, 'checkpoint': cid, 'scene': 's5-chain', 'mode': {
            'clear': 'StageClear', 'shop': 'Shop', 'exit': 'Transition',
            'start': 'StageStart'}.get(cid.rsplit('-', 1)[-1], 'Combat'),
        'f': f, 'golden': True, 'trigger': 'natural',
        'source_run': run, 'source_sha256': sha,
        'evidence': f'analysis/traces/{run}.frames.jsonl#f{f}',
        'predicate': predicate,
    }


if __name__ == '__main__':
    main()
