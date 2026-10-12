# oracle/subroutines/ —— L2 子程序级差分用例

**期望 = ROM 执行结果，实际 = game-core 同名函数**；三方全等才算绿：
`case.out（golden，手推自 A 级反汇编语义） == 6502 核执行 ROM 原字节 == game-core::l2ports`。

运行：

```bash
python3 tools/diff_subroutines.py        # subroutine-diff 门禁入口（薄封装）
cargo test -p game-oracle --test l2_cases  # 同一差分引擎（rust-tests 门禁内）
```

## case 格式（JSONL，一行一条）

```json
{"id":"…","subroutine":"…","entry":"bankNN:AAAA","stops":["bankNN:AAAA",…],
 "context":{"r6":N,"r7":N},"scratch":["0004",…],
 "in":{"regs":{"a","x","y","p"},"ram":{"hexaddr":byte}},
 "out":{"regs":{"a","x","y"},"writes":{"hexaddr":byte}}}
```

- `entry`/`stops` 为 CPU 地址 + bank 标签（bank 上下文按 t2 布局：$8000 窗=R6、$A000 窗=R7、
  $C000=bank14、$E000=bank15 固定，见 `docs/findings/rom-forensics.md` §2）；
  `stops` 为空 ⇒ 调用约定执行（预压哨兵返回地址，跑到 RTS 收集出参）。
- `scratch` = 被调用者内部暂存单元（如 ED03 的 $04-$06），case 显式声明后从两侧写集剔除
  （可见的比对面裁剪；栈压入/弹出恒不入写集——但栈页里的**游戏变量**照常记录，如 $018E/$018F）。
- 差分面 = regs{a,x,y} + 写集；P 为执行副产物不入比对面（输入 P 影响分支，仍进 `in`）。

## 用例覆盖（54 条，先写用例再实现）

| 子程序 | ROM 位置 | 证据 | 用例 |
|---|---|---|---|
| player_x_clamp_hi/lo | bank03:A0D0-A0E8 | N-0011（钳 $0E/$F3 + 触钳亚像素归零） | x-hi-1..4 / x-lo-1..3 |
| player_y_clamp_hi/lo | bank03:A09D-A0B7 | N-0012（钳 $14/$DF） | y-hi-1..3 / y-lo-1..2 |
| digit_conversion | bank00:9DE8-9DF6 | N-0023（/10 取位一步） | digit-1..5 |
| consumption_lookup | bank03:A2C4-A2CF | R-fire-3 回退档（消耗表 [1,2,3,2]） | cons-1..7 |
| bomb_drop（R-fire-2/3/4 全链） | bank03:A2B4-A2FA + JSR $ED03 | N-0017、R-fire-2/3/4、C-bomb-consume/C-bomb-type-tbl | bd-1..9 |
| movement_tick（R-move-5/6 积分+钳制） | bank03:A08D-A0EB | N-0011..0016、R-move-1..6（含 Y 溢出 quirk） | mt-01..14 |
| pickup_handler（同型升级/异型切换/回卷） | bank2:$922B-$9258 | N-0032/33、R-fire 系（含 $0C 非上限 quirk） | pk-1..7 |

数值表（A 级，`docs/spec/constants.md` v2）：消耗量 `$A336-$A339`=[1,2,3,2]、
类型 `$A33A-$A33D`=[2,3,4,4]（**登记不解释**，语义=H-11 待证）；`$018F` 为压缩 BCD，
ED03 扣减 = 逐半字节借位减（bank15:ED03-ED26）。

> **采样面教训（t38，$0100 页盲区）**：游戏变量并不都在零页——`$018F/$0190`（弹药/弹种）
> 落在 **$0100 页（栈页游戏变量区）**，夹在 zp($0000-$00FF) 与 iram($0300-$07FF) 两个 watch
> 区间之间；t31 曾跨页误映射到 $008F/$0090 计数器。**watch 覆盖必须按变量地址逐个核对所属页**，
> 需要新字段先走证据轨道变更流程（t38 已批 +3 watch）。相关：t24 栈页剔除教训（write_stack 通道）。

纪律：改用例 = 改证据轨道，单独 commit + Evidence/Confidence（AGENTS.md §9）；
差分红了按 §4 定位层（移植错→porter / 规格缺口→spec-writer / 投影错→decomp-analyst），
禁止调阈值放水。
