# STG-07 敌波时间线解码（差异面 — 八关横切素材第 7 关）

> 【2026-10-15】captain 直办（decomp-analyst 回合基建连断；机器出表=工具可重导，语义面按
> stg02-timeline §6 交接口径同型）。B1 §4.1 流清单：STG-07 op5 流仅 1 条（$AF7F@$0295）✓ 复核。

## ① 机器出表（可重导：`python3 tools/wave_stream_decode.py wave 7`）

- **波次流 $AF7F**（脚本 thr=$0295）：C9=$FB（bit0→far-call bank11:$8012/bit1→速度快照——模板全变体）；
  **CA=$B1**（bit0→$26|关末门臂 ✓ 不变式；**bit4/5/7 置位=八关新形态**）；
  速度字节=$07/$7F=$02；**写表1=$AF96 非零**（S2-5 同族）；清单 count=1 types=[$B9]（新型值）；
  $D1=$1E/$1E（同值对）；ED3B=$0B/$0F/$A5；DB4C=$13；ED6C=零对；next=$B901；C9 bit2=0 链止
- **调度流**（节选）：[1]T=$2E/[2]T=$2F 四参对型（模板恒定）；+609t 多 item 双 [1]T=$2E；
  **+745/+747t ($B7,$07)/($B7,$87) pair 项**（ctrl pair 形态=八关首次实测）；尾自环 @AF79（+747t，模板恒定）

## ② 与 STG-02..06 差异面对照

| 面 | STG-07 | 横切族（已积累） |
|---|---|---|
| C9 流1 | $FB（全变体） | {$F2, $E2, $FB} |
| **CA 流1** | **$B1（新形态 bit4/5/7）** | {$00, $70, **$B1**} |
| 清单 | 1×[$B9]（新 type） | {1×, 2×}；type 值族扩 |
| 多 item | 双 [1]T=$2E ×1 | {2, 3, 18, …} |
| **pair 项** | **($B7,$07)/($B7,$87) 首实测** | ctrl pair 形态补全 |
| 写表 | 写表1 非零（$AF96） | {零, 单, 双, 非零对} |
| LUT 邻接 | （随流数=1） | {+9, +4, +11, +4, +8, +3, …} |
| 尾自环 | @AF79 | 模板恒定 ✓ |

## ③ 特异挂账
- S7-2：CA=$B1 bit4/5/7 位义（归 S3-3 横切族一起裁）
- S7-3：pair 项 ($B7,$xx) 字段语义（与 ctrl pair 跳表项对账——B1 §1.4 项格式 "ctrl（1/11=定型…）" 的 pair 分支实测首例）
- 型值 $B9/$B7 不命名（未证不发明）

## ⑥ 复现命令
```
python3 tools/wave_stream_decode.py wave 7
python3 tools/wave_stream_decode.py spawner 7
python3 tools/wave_stream_decode.py evt 7
python3 tools/wave_timeline.py sched   # STG-07 段
```
