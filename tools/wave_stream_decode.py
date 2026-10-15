#!/usr/bin/env python3
"""wave_stream_decode.py — 波次记录流（§1.4 字段格式）+ 生成器流（§1.5 格式）逐记录解码器。

证据口径（docs/findings/enemy-timeline.md §1.4/§1.5 + 读点源码钉死）：
  波次记录（23B，(($CC),Y 读点逐字节对账 bank00:897F-8D36 / 8A97-8B10）：
    0x00 $C9   标志：bit0→far-call bank11:$8012（89D5）；bit1→速度快照 $191-195（89A1-89C7，
               Y=0x02 字节加入 $194/95，Y=0x02==0 时 $80-$83 清零+$CB|=2）；bit2→跳转下记录（8D17-8D36）
    0x01 $CA   标志：bit0→$26|=关末门（8CC9-8D0C 消费链，G-S5-6）；bit2→出敌清单按 type 匹配已有槽（8ADE）
    0x02       速度/滚动字节（bit1 时消费）
    0x03       $7F
    0x04/0x05  写表指针1（8A97 LDY #4 → sub_8D8D）
    0x06       →$ED3B 参数
    0x07/0x08  出敌清单指针 → $14/$15（8AA0 LDY #7；lo=0x07/hi=0x08）
    0x09       $D2（8B08 LDY #9）
    0x0A/0x0F  $D1
    0x0B       →$ED3B
    0x0C/0x0D  写表指针2
    0x0E       →$DB4C
    0x10/0x11  →$ED6C 组1
    0x12/0x13  →$ED6C 组2
    0x14       →$ED3B
    0x15/0x16  下记录指针（$C9 bit2 时：8D1D LDY #$15，lo→$CC/hi→$CD）
  出敌清单 = [count][type×count]（8A9C-8B06：($14),0→count→$11，逐字节 type→$400+槽）
  生成器流（loader B，bank00:83E7-8444）：
    p0=0 流止；p0≥$80 延时（$A3:=p0&$7F，每 tick DEC）；p0<$80 且 p1 bit3=0 ⇒ 4B 记录
    {X=p0−$10（carry 语义 en-ld-1）, p1: bit0-2=VM 槽/bit3=停用/bit4-7=循环计数 $C2,
     p2/p3=行为脚本指针→$197/$19D,X}；p1 bit3=1 ⇒ 停用记录 2B。

用法：
  python3 tools/wave_stream_decode.py wave [N]     # STG-N op5 波次记录流（默认 2）
  python3 tools/wave_stream_decode.py spawner [N]  # STG-N 生成器流（默认 2）
  python3 tools/wave_stream_decode.py evt [N]      # STG-N 事件机流 hex+控制字节族扫描（默认 2）

复现纪律：解码输出可由本命令逐字节重导；语义列只引读点代码，不做命名发明。
"""
import sys

ROM = "rom/gunnac_cn.nes"
SCRIPT_OPS = {0: 3, 1: 4, 2: 8, 3: 2, 4: 3, 5: 2, 6: 2, 7: 2, 8: 2, 9: 2, 10: 2, 11: 1}


def fb(bank, addr):
    return 0x10 + bank * 0x2000 + (addr - 0xA000)


def load():
    return open(ROM, "rb").read()


def stage_entry(d, n):
    e = d[0x10010 + n * 8:0x10010 + n * 8 + 8]
    return dict(script=e[0] | e[1] << 8, bank=e[5], lut=e[6] | e[7] << 8)


def script_streams(d, n):
    """脚本流头部解析：返回 dict（evt/spawner/sched/waves=[(rec,ptr,thr)]/waves_loaded）。"""
    st = stage_entry(d, n)
    off = fb(st["bank"], st["script"])
    out = dict(evt=None, spawner=None, sched=None, waves=[])
    p = off + 2
    for _ in range(256):
        op = d[p]
        if op > 11:
            break
        ln = SCRIPT_OPS[op]
        args = d[p + 1:p + 1 + ln]
        thr = d[p + 1 + ln] | d[p + 2 + ln] << 8
        if op == 1:
            out["evt"] = args[0] | args[1] << 8
            out["spawner"] = args[2] | args[3] << 8
        elif op == 2:
            out["sched"] = args[0] | args[1] << 8
        elif op == 5:
            out["waves"].append((args[0] | args[1] << 8, thr))
        p += 1 + ln + 2
        if op in (3, 7):
            break
    out["bank"] = st["bank"]
    out["lut"] = st["lut"]
    return out


# ---- 波次记录（23B）----
RECLEN = 0x17


def decode_wave_record(d, bank, addr):
    o = fb(bank, addr)
    f = list(d[o:o + RECLEN])
    rec = dict(addr=addr, raw=f)
    rec["c9"] = f[0x00]
    rec["ca"] = f[0x01]
    rec["speed_byte"] = f[0x02]
    rec["f7f"] = f[0x03]
    rec["wtab1"] = f[0x04] | f[0x05] << 8
    rec["ed3b_a"] = f[0x06]
    rec["list"] = f[0x07] | f[0x08] << 8
    rec["d2"] = f[0x09]
    rec["d1_a"] = f[0x0A]
    rec["ed3b_b"] = f[0x0B]
    rec["wtab2"] = f[0x0C] | f[0x0D] << 8
    rec["db4c"] = f[0x0E]
    rec["d1_b"] = f[0x0F]
    rec["ed6c1"] = f[0x10] | f[0x11] << 8
    rec["ed6c2"] = f[0x12] | f[0x13] << 8
    rec["ed3b_c"] = f[0x14]
    rec["next"] = f[0x15] | f[0x16] << 8
    # 出敌清单
    if 0xA000 <= rec["list"] < 0xC000:
        lo = fb(bank, rec["list"])
        cnt = d[lo]
        rec["list_count"] = cnt
        rec["list_types"] = list(d[lo + 1:lo + 1 + cnt])
    else:
        rec["list_count"] = None
        rec["list_types"] = []
    return rec


def cmd_wave(d, n):
    st = script_streams(d, n)
    print(f"== STG-{n:02d} op5 波次记录流（bank{st['bank']}；§1.4 字段格式 23B/记录）==")
    for ptr, thr in st["waves"]:
        print(f"\n-- 流 ${ptr:04X}（脚本 thr=${thr:04X}）--")
        seen = set()
        addr = ptr
        for i in range(16):
            if addr in seen:
                print(f"  @${addr:04X} [循环回已见解] 停")
                break
            seen.add(addr)
            r = decode_wave_record(d, st["bank"], addr)
            c9, ca = r["c9"], r["ca"]
            flags = []
            if c9 & 0x01:
                flags.append("bit0→far-call bank11:$8012")
            if c9 & 0x02:
                flags.append("bit1→速度快照")
            if c9 & 0x04:
                flags.append("bit2→跳转")
            ca_f = []
            if ca & 0x01:
                ca_f.append("bit0→$26|关末门")
            if ca & 0x04:
                ca_f.append("bit2→清单 type 匹配重指派")
            lt = ",".join(f"${t:02X}" for t in r["list_types"]) if r["list_types"] else "—"
            print(f"  @{r['addr']:04X} C9=${c9:02X}({' '.join(flags) or '—'}) "
                  f"CA=${ca:02X}({' '.join(ca_f) or '—'})")
            print(f"        速度字节=${r['speed_byte']:02X} $7F=${r['f7f']:02X} "
                  f"写表1=${r['wtab1']:04X} 写表2=${r['wtab2']:04X}")
            print(f"        清单=${r['list']:04X} count={r['list_count']} types=[{lt}] "
                  f"$D2=${r['d2']:02X} $D1=${r['d1_a']:02X}/${r['d1_b']:02X}")
            print(f"        ED3B=${r['ed3b_a']:02X}/${r['ed3b_b']:02X}/${r['ed3b_c']:02X} "
                  f"DB4C=${r['db4c']:02X} ED6C=${r['ed6c1']:04X}/${r['ed6c2']:04X} "
                  f"next=${r['next']:04X}")
            if c9 & 0x04:
                addr = r["next"]
            else:
                print(f"        （C9 bit2=0：不跳转，链止）")
                break


# ---- 生成器流（loader B）----
def cmd_spawner(d, n):
    st = script_streams(d, n)
    ptr = st["spawner"]
    bank = st["bank"]
    o = fb(bank, ptr)
    print(f"== STG-{n:02d} 生成器流 ${ptr:04X}（bank{bank}；§1.5 loader B）==")
    tick = 0
    p = o
    rec = 0
    while True:
        p0 = d[p]
        if p0 == 0:
            print(f"  @{p - o + ptr:04X} +{tick:5d}t : [流止 00]")
            break
        if p0 >= 0x80:
            dl = p0 & 0x7F
            print(f"  @{p - o + ptr:04X} +{tick:5d}t : [延时 ${p0:02X}={dl}t]")
            tick += dl
            p += 1
            continue
        p1 = d[p + 1]
        if p1 & 0x08:
            x = (p0 - 0x10) & 0xFF
            print(f"  @{p - o + ptr:04X} +{tick:5d}t : [停用 2B] X=${x:02X} p1=${p1:02X}")
            p += 2
            rec += 1
            continue
        x = (p0 - 0x10) & 0xFF
        slot = p1 & 0x07
        loops = (p1 >> 4) & 0x0F
        beh = d[p + 2] | d[p + 3] << 8
        print(f"  @{p - o + ptr:04X} +{tick:5d}t : [记录4B] X=${x:02X} VM槽={slot} "
              f"循环={loops} 行为指针=${beh:04X}")
        p += 4
        rec += 1
        if rec > 512:
            print("  [截断] 512 记录上限")
            break


# ---- 事件机流（loader A）：控制字节族扫描 ----
def cmd_evt(d, n):
    st = script_streams(d, n)
    ptr, bank = st["evt"], st["bank"]
    o = fb(bank, ptr)
    print(f"== STG-{n:02d} 事件机流 ${ptr:04X}（bank{bank}；loader A 控制字节族扫描）==")
    print("（格式未全解——Wt/Wv 遗留：此处只登记 $FC-$FF 控制字节分布与 hexdump，不发明语义）")
    sp = st["spawner"]
    region = d[o:fb(bank, sp)]
    print(f"hex（$EVT..$SPAWNER 共 {len(region)}B）：")
    for i in range(0, len(region), 16):
        ch = region[i:i + 16]
        print(f"  ${ptr + i:04X}  {ch.hex(' ')}")
    ctrl = {}
    for b in region:
        if b >= 0xFC:
            ctrl[b] = ctrl.get(b, 0) + 1
    print("控制字节族计数（≥$FC）：", {f"${k:02X}": v for k, v in sorted(ctrl.items())})


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return
    d = load()
    cmd = sys.argv[1]
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 2
    {"wave": cmd_wave, "spawner": cmd_spawner, "evt": cmd_evt}[cmd](d, n)


if __name__ == "__main__":
    main()
