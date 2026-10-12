# RAM watch 语义考证第一批（t18）— 玩家坐标/金钱/HUD/弹数 + t_002/t_007 修正

> 纪律（AGENTS.md §3）：每条结论分 **已证实 / 假设**；trace + RAM diff = A 级。
> 动态证据：t12 场景产物 `analysis/traces/gameplay-stage1.frames.jsonl`（逐帧 zp+$0320 watch）、
> `analysis/traces/*_probe.ram.hex`（60 帧间隔全 RAM $0000-$07FF）、`oracle/scenarios/gameplay-stage1.txt`
> （hold/press_every 输入段定义）。静态交叉：`reference/seed/gunnac.asm`（IDA 冻结 listing）+
> `analysis/ghidra/bank00-15.c`。命名登记：`data/naming/ledger.jsonl` N-0011…N-0027。

## 1. 玩家坐标（①）— 已证实（A 级）

**移动例程**（seed `bank03:A084-A0E8`，bank2/3 游戏逻辑单元；Ghidra bank03.c 交叉）：

| 符号 | 地址 | 证据（代码 + RAM diff 双向） |
|---|---|---|
| `player_x` | **$04E0** | `bank03:A0E5 STA $04E0`；钳制常量 `$0E/$F3`（`bank03:A0D0-A0E3`）；probe diff：right 段 184→**243(=$F3 钳)**、left 段→**14(=$0E 钳)**、down/up 段恒 128 |
| `player_y` | **$04FC** | `bank03:A0B2 STA $04FC`；钳制常量 `$14/$DF`（`bank03:A09D-A0B0`）；probe diff：down 段 160→174→**223(=$DF 钳)**、up 段递减 223→54→37、X 段恒 160 |
| `player_x_subpix` | **$0470** | `bank03:A0C2 ADC $0470 / A0E8 STY $0470`；钳制分支 `A0E3 LDY #0` 归零（RAM 瞬态 64/96 一致） |
| `player_y_subpix` | **$048C** | `bank03:A091 ADC $048C / A0B5 STY $048C`；`A0B0` 归零 |
| `player_x_velocity` | **$0550/$056C**（亚像素/有符号整数） | `bank03:A0BE-A0CB`；probe：right 段 (208,2)=**+2.8125 px/帧**、left 段 (48,253)=**−2.8125**、无输入段 (0,0) |
| `player_y_velocity` | **$0588/$05A4** | `bank03:A08D-A098`；down (208,2) / up (48,253) / 无输入 (0,0) |
| `bomb_spawn_x/y` | **$04E1 / $04FD** | `bank03:A2E9-A2F2` 投放时刻快照 `LDA $04FC/STA $04FD`、`LDA $04E0/STA $04E1` —— 反向印证坐标语义 |

- hold 段 vs RAM 变化相关性：6 段方向（right/left/right/down/up/right）全部符号吻合；
  亚像素对速度恒 ±2.8125 与像素累加轨迹一致（60 帧 = 168.75px ≈ 实测段位移）。
- 复现：`python3 - <<'EOF'` 载入 `gameplay-stage1_f*_probe.ram.hex` 按段取 `ram[end][0x4E0]-ram[start][0x4E0]`，
  对照 `oracle/scenarios/gameplay-stage1.txt` 的 `hold` 段；seed 对照
  `grep -n "STA     \$4E0\|STA     \$4FC\|CMP     #\$F4\|CMP     #\$E0" reference/seed/gunnac.asm`。

### 假设（B/C 级）

- 速度对的「亚像素/整数」分划（$0550 为 1/256 分数、$056C 为有符号像素）—— 由 (208,2)/(48,253) 数值反推，B 级。
- up 段结束后 Y 自动回到 160（f2160=37 → f2220=160）—— 疑为死亡重生或自动回中，**C 级**，S2 复现。

## 2. 金钱/道具计数（②）— 负结果存档 + 假设

### 已证实（A 级：负结果本身）

- gameplay 画面顶部数值 f2160=**4400** → f2280=**7200**（截图 `shots/gameplay-stage1_f0{2160,2280}_probe.png`）。
- **该数值不在内部 RAM**：对两帧 `*.ram.hex` 全 $0000-$07FF 穷举 16 位 LE/BE、BCD 正/反序、
  4 位数字任意 stride(1-8)、÷10/÷100 缩放共 12 种编码 —— **全部无匹配**（复现脚本见 §5）。
- `digit_conversion_scratch = $018E`（B 级）：`bank00:9DE8-9DF7` 连减 10 取十进制位循环；
  写入点 `bank00:9C86`（价格 10/30，按 `$FFF9` 版本标志二选一 —— 与 S0 `$FFF9` 发现呼应）。

### 假设（B/C 级）

- 显示源在 **WRAM $6000-$7FFF**（MMC3 PRG-RAM；t12 的 RAM watch 只覆盖内部 2KB）→ `_todo_money_display_src`，S2 扩 watch 验证。**C 级**。
- 顶部数值是「分数」还是「金钱」（`你有──￥` 同款）未定 —— 二者在本场景都单调增。**C 级**。

## 3. HUD 状态地址（③）— 部分已证实

### 已证实（A 级）

- **`bomb_ammo` = $018F**：HUD 行 `F3`→`F4` 与 `$018F: 3→4` 帧间差分同步（f2160→f2280）；
  投放例程 `bank03:A2BF-A2E6`：`LDA $018F/BEQ` 退出、`CMP $A336,Y` 消耗量表、`JSR $ED03` 扣减回写；
  初始化 `bank00:9C5D STA $018F = 4`。
- **`weapon_type` = $33**（B 级）：`bank15:E143-E150` 五值域钳制（`AND #$07/CMP #$05`）与 E-W02 五主武器一致；
  HUD 首行字母（A）与之对应（B 级绑定）。
- **`controller_held` = $18 / `controller_pressed` = $16**：逐帧 watch 与输入段一一对应
  （bit0=R,1=L,2=D,3=U,7=A；$16 为边沿）。投放例程 `bank03:A2B4 LDA $16/AND #$80` 边沿触发。

### 假设（B/C 级）

- HUD 行 1『A0』= 武器字母 + 等级数字；等级 var 未定位 → `_todo_hud_weapon_line_src`。**C 级**。
- HUD 行 3『1』= 残机数；var 未定位。**C 级**。
- 『红圈数字』= 掉落物（数字圈=E-W02 武器、字母圈=E-W04 炸弹），载体为 object slot 区
  （$04xx 步长 0x1C）类型字段，具体 slot 偏移未定 → `_todo_item_drop_slots`。**C 级**。

## 4. 弹数/武器槽（④）— 已证实（A 级）

- **炸弹弹数 `bomb_ammo`=$018F** 见 §3（投放消耗 + HUD 显示双向印证）。
- **投放例程全貌**（`bank03:A2B4-A2FA`）：A 边沿（$16 bit7）→ `$0401==0` 校验 → `$018F` 非零校验 →
  按 `$3B` 索引消耗量表 `$A336,Y` 与炸弹类型表 `$A33A,Y`（类型写 `$0765`）→ 扣弹 →
  坐标快照 `$04E1/$04FD` → `$0401:=7`（`game_action_mode`，N-0024）。
- **武器类型 `weapon_type`=$33**（§3）；武器槽显示/等级对应关系未闭环（假设区）。
- 主武器弹药无限（E-W02）与「press_every 12 帧 A」段内无递减计数的观测一致
  （逐帧 zp 扫描无 12 帧周期递减项）—— 反向印证 E-W02，**B 级**。

## 5. 复现命令附录

```bash
# 输入段定义（hold/press_every 帧号）
sed -n '/hold\|press_every/p' oracle/scenarios/gameplay-stage1.txt
# 逐帧 zp watch（含 $18/$16 输入位与坐标反推）
python3 -c "import json;[print(json.loads(l)['f'],json.loads(l)['w']['zp'][0x36:0x38]) for l in open('analysis/traces/gameplay-stage1.frames.jsonl')][:5]"
# 段位移核对（player_x/y 钳制点）
python3 - <<'EOF'
import glob,re
fs={int(re.search(r'_f(\d+)_',p).group(1)):p for p in glob.glob('analysis/traces/gameplay-stage1_f*_probe.ram.hex')}
r=lambda f: bytes.fromhex(open(fs[f]).read().strip())
print('right1 X:',r(1320)[0x4E0],'->',r(1440)[0x4E0],'| left X ->',r(1560)[0x4E0],'| down Y:',r(1860)[0x4FC],'->',r(1920)[0x4FC])
EOF
# 代码交叉（钳制常量/STA 落点）
grep -n "STA     \$4E0\|STA     \$4FC\|CMP     #\$F4\|CMP     #\$E0\|LDA     \$18F\|STA     \$18F" reference/seed/gunnac.asm
# 金钱负结果复现（12 种编码穷举）
#   见 docs/findings/ram-watch-s3.md §2 所述：对 f2160/f2280 两帧全 RAM 执行
#   LE16/BE16/BCD 正反/逐位 stride1-8/÷10/÷100 检索 —— 全部空集
```

## 6. t13 遗留修正（t_002 / t_007）

> 【2026-10-12 更正】对 docs/findings/data-tables.md §5 的 t_002/t_003/t_004/t_007 行原地更正：

- **t_002（bank2 $8678-$9FFF）重分类 = mixed_code**：种子反汇编 branch-align **84.3%**（880/1044 分支目标落指令边界），
  头部为真指令流（`JSR $C304 / BCS / JMP $C430 / RTS`…）。移出数据面分母。
- **t_003（bank5 $8F8D-$9FFF）重分类 = mixed_code**：branch-align **92.0%**；头部数据岛
  `34 30 2C 28…` 后紧随真代码（含 `STA $04FC` 玩家 Y 初始化）。t13 的「数值曲线表」猜测就此**证伪**（曲线段是代码前的数据岛）。
- **t_004（bank6 $93A8-$9FFF）重分类 = mixed_code**：branch-align **97.1%**。
- **t_007（bank15）边界修正**：run 截断至 **$FFFA 之前**（`$FD2A-$FFF9`），向量表 $FFFA-$FFFF 移出
  （xref 计数 8→7）；t13「边界误吞向量表」修正完成。
- 判定规则（机械、可复现）：xref 目标播种线性反汇编 → 条件分支目标落指令边界比例
  （纯数据 43-77% vs 真代码 84-97%，阈值 80%）→ `tools/remap/run.py` 内实现，
  产物 `data/naming/remap-table.json` 的 `mixed_regions` 列表。

## 7. 遗留假设清单

| # | 假设 | 级 | 去向 |
|---|---|---|---|
| W1 | 显示源（4400→7200）在 WRAM $6000-$7FFF | C | S2 watch 扩展 |
| W2 | 顶层数值 = 分数 or 金钱 | C | S2 收支事件相关性 |
| W3 | HUD『A0』的等级 var、『1』的残机 var | C | S2 HUD 帧差分 |
| W4 | 红圈掉落物类型字段 slot 偏移 | C | S2 object slot trace |
| W5 | up 段后 Y 自动回中的机制 | C | S2 复现 |
| W6 | 速度对亚像素/整数分划 | B | S1 反汇编速度查表 ($A30B/$A2FB) |

---

## 8. t22 追加：金钱 WRAM 定位 + HUD 补证（H-11/H-12 收口）

> 【2026-10-12 更正（t22）】§2 负结果与 W1 假设**证伪**，详见本节；原文保留不删（证伪更正文化）。

### 8.1 金钱显示源 —— 已证实（A 级）：`money_counter_bcd = $E0-$E6`

**t18 负结果的根因（更正）**：①端点配对错误 —— 并行 read_image 结果错序致 f2280 显示值误读为 7200，
像素位图解码实为 **6800**（RAM 为准，D3）；②编码误设 —— 显示尾位 "0" 是**真实数位**（7 位 BCD 的最低位 0），
不是字面量后缀，故 12 种"4400→7200 相邻编码"搜索全部落空。

**正解（多证据链）**：

| 事实 | 证据 |
|---|---|
| `$E0-$E6` = 7 位 BCD 计数器，值 = 显示数值 | probe 实测 f2160=0,0,0,4,4,0,0=**4400**；f2220=**5700**；f2280=**6800**；f2340=**7200** —— 与屏幕位图逐帧吻合 |
| 显示渲染 = 精灵数字 + 前导零抑制 | `bank14:D009-D046`：`LDA $E0,Y`（Y=0..6）→ `ASL A / ORA #$01` tile 码 → `STA $0201,X`；`LDA $05 / BNE` 前导零跳位；`CPY #$07` 七位 |
| 入账（加法器） | `bank14:DBCE-DBFF`：`LDA $DCCF,Y`（事件类型×6 → 6 位 BCD 加数表）逐位 `ADC $E0,X` + `CMP #$0A` 进位；溢出封顶 999999 |
| 事件分发 | `bank14:DB4C`（A=奖励类型；$26&$88 状态屏蔽；`JSR $DBCE` 后 `JSR $DAE7` 阈值检查）；调用点 bank00:8C86、bank02:8660 |
| 金钱定性 | shop 场景全程 000000 ↔ 『你有──￥0』（E-S02）一致；E-W06 击破掉落+商店消费语境（B 级绑定） |
| **W1 证伪** | WRAM $6000-$7FFF watch 全程 open-bus 恒 0x60（2400 帧零变化）⇒ PRG-RAM 未映射/未用；金钱在内部 RAM |

### 8.2 extend 阈值 —— 已证实（B 级）：`extend_threshold_bcd = $E7-$EC`

- `bank14:DAE7-DAFC`：逐位 `CMP $E7,Y` 与计数器比较，达标置 `$ED=$40`（extend 触发旗标）。
- probe 实测阈值恒 `0,0,8,0,0,0` = **8000**（计数器 7200 逼近下一档）。

### 8.3 H-12 收口：HUD 字母归因 —— 已证实（A 级）：`bomb_type_index = $0190`

- 初始化：`bank15:E138-E13D` `LSR×3 / AND #$03 / STA $0190`（0-3 = 四弹种）。
- 拾取事件链（f2160→f2280 probe diff）：f2220 拾取字母圈 **W** → `$0190: 0→3`、HUD 字母 **F→W**、
  `$018F` 弹数 **3→4** 三者同步 —— **字母切换归因 = 当前弹种索引**（H-12 收口）。
- 索引↔字母映射：**0=F、3=W**（实测）；**1=B、2=T**（按 E-R08 `F B T W` 串序推定，B 级）。
- 『红圈数字』= 掉落物拾取源（字母圈→弹种切换实测成立；数字圈=E-W02 武器）；具体 object slot 偏移仍留 W4。

### 8.4 假设区更新（原地）

| # | 假设 | 级 | 状态 |
|---|---|---|---|
| W1 | 金钱在 WRAM $6000-$7FFF | ~~C~~ | **【2026-10-12 证伪】** WRAM open-bus；真源 $E0-$E6 |
| W2 | 顶层数值 = 分数 or 金钱 | C | 倾向金钱（shop ￥ 显示同值 + E-W06），独立『分数』显示未见 —— 维持观察 |
| W3 | HUD『A0』等级 var、『1』残机 var | C | 未闭环（本场景无变化帧，无差分机会） |
| W4 | 红圈掉落物 object slot 偏移 | C | 拾取效果链已实测（W4a），slot 字段偏移未定位 |
| W6 | 速度对亚像素/整数分划 | B | 维持 |
| W7 | 显示字形读图误差（6700/6800）教训：**并行 read_image 结果序不可信**，像素位图/RAM 为准 | — | 方法论 gotcha（建议入 docs/knowledge） |

### 8.5 复现命令

```bash
# watch 扩展（观察性 diff）：oracle/scenarios/{gameplay-stage1,shop}.txt 尾部
#   watch_range 0300-07ff iram / watch_range 6000-7fff wram
python3 tools/scenario_run.py run gameplay-stage1 && python3 tools/scenario_run.py run shop
# 计数器/阈值时间线（probe RAM）
python3 - <<'EOF'
import glob,re
fs={int(re.search(r'_f(\d+)_',p).group(1)):p for p in glob.glob('analysis/traces/gameplay-stage1_f*_probe.ram.hex')}
for f in sorted(fs):
    if f>=2100:
        r=bytes.fromhex(open(fs[f]).read().strip())
        print(f,[r[0xE0+i] for i in range(7)],[r[0xE7+i] for i in range(6)],'type$190=',r[0x190],'ammo$18F=',r[0x18F])
EOF
# 显示渲染/加法器/阈值代码
grep -n "STA     \$190\|LDA     \$E0,Y\|CMP     \$E7,Y\|JSR     \$DBCE\|LDA     \$DCCF,Y" reference/seed/gunnac.asm
```

---

## 9. t25 追加：武器拾取/受击残机/A 键分工（H-01/H-02/H-14/H-15/W3）

> 场景产物：`oracle/scenarios/{akey-role,weapon-pickup,hit-test}.txt`（新 DSL 场景，watch 覆盖 zp+iram+wram）。
> 【2026-10-12 更正（t25）】N-0022『weapon_type=$33』**证伪撤回**，见 §9.1。

### 9.1 武器拾取/升级（H-01/H-02 收口）— 已证实（A 级）

**拾取处理例程**（bank2:$922B-$9258 = PRG 0x0523B，seed 线性扫描漏解、raw 字节 `85 33/85 34` 定位补出）：

```
$922B: CMP $34        ; 拾到的圈上编号 vs 当前武器槽
$922D: BNE $9258      ; 不同型 → 切换
$923D: 同型 → INC $33 ; 等级 +1（bank2:$924D，回卷置 $0C）
$9258: STA $34        ; 异型 → 武器槽 = 新编号
```

| 结论 | 证据 |
|---|---|
| `weapon_slot = $34`（圈上编号，0 起） | `CMP $34/BNE` + `STA $34` 切换分支；初始化 bank2:$9222=0；死亡重置 bank03:$A376=0 |
| `weapon_level = $33`（同型连收计数） | 同型分支 `INC $33`（bank2:$924D）；初始化 bank2:$9220=0；重置 bank03:$A378=0 |
| **H-02 两说合一**：异型圈=切换武器、同型圈连收=升级 | E-W03 的两种说法各对应一个分支：『连收同型圈升级』= INC $33 ✓；『Power P』= 同型升级道具的叙述变体（道具语义另考） |
| HUD『A0』解码 | 字母位=weapon_slot（0→'A'）+ 数字位=weapon_level（t22 位图一致） |
| 射击模式按槽选择 | bank03:$A1F5 `LDA $34/BNE`：0=六型自动连射、1→type3 齐射、2→type4、3→退出、>3→type6（H-01 槽位-弹型对应骨架） |

### 9.2 A 键分工（H-14）— 静态定谳（A/B 级）+ 动态旁证

- **主武器 = 计数器驱动自动连射**：射击生成器 bank03:$A1FD-$A22B（六弹型循环 `LDA $072E,X/CMP $074A,Y`
  到点生成弹对象，源坐标=$04E0/$04FC）**全程无按键测试**（A/B 级静态）。
- **A 边沿 = 投放**：bank03:$A2B4 `LDA $16/AND #$80/BEQ` → 消耗 $018F（t22 N-0017/N-0030 链）。
- **B 边沿**：bank03:$A193/A1A9 `AND #$40`（弹种切换相关，E-R06『B键转换』呼应）。
- 动态旁证：akey-role 场景 hold-A 100 帧内主武器计数器 $072E,X 无 A 相关脉冲；投放窗口与 $0401=0 窗口吻合（§9.3）。
- **结论**：A=投放/特殊键，主武器自动连射 —— H-14 收口（press_every 段『出弹』系自动连射与 A 同期，非 A 驱动）。

### 9.3 $0401 值域（H-15）— 观测补证（B/C 级）

- gameplay 实测值域 **{7, 0}**：akey-role 全程 7；0 窗口 f1460-1467 / f1676-1707 / f1916-1923 / f2132+（周期 ~220 帧）。
- 投放例程要求 `$0401==0`、完后置 7 → 疑『投放冷却/在途态』（7=不可投、0=可投）→ `_todo_401_deploy_semantics`（N-0036，C 级）。
- 商店上下文选择光标语义维持（N-0003/N-0024 多用途注记）。

### 9.4 受击/残机（W3）— 负结果存档（t18 纪律）

- hit-test 场景（静止受击设计）**未进入实机段**（f1260 起无 HUD/坐标变化，位图显示非战斗屏）——
  标题→商店→退出路径存在**run 间分岔**（三次运行同前缀、命中不同结局；疑 RNG/时序分岔）。
- 残机计数 var **未定位**（无受击事件样本）；W3 维持 `_todo_`。
- 翼装 wing 双击语义（E-W05）无样本，维持假设区。
- 复现缺口已用场景脚本固化（hit-test.txt 可重跑），建议后续批次加宽退出路径容差（多点几次 B）。

### 9.5 环境/方法注记

- **W8（新）**：akey-role 运行出现无排程的方向位（$18 bit0/1 与 weapon-pickup 的真实 hold 段一致）——
  疑实机开场存在**游戏内虚拟输入序列**（自演示进弹）写入同一输入 RAM；hit-test 因未进实机不出现。
  未定谳（C 级）；后续以『无输入实机段』对照复核。
- 并行任务于 15:31 重跑 gameplay-stage1（外部 actor）：分析中避免引用被并发重写的 trace 文件；
  本节结论仅取 akey-role/weapon-pickup/hit-test（本轮自有产物）与 seed 静态证据。
- 【2026-10-12 更正】N-0022 撤回：t18 对 bank15:E143-E150 的『五值域』系误读
  （`CMP #$05/BCC` 分支跳过 STA，非武器域；开机随机项写 12-14）。

### 9.6 复现命令

```bash
python3 tools/scenario_run.py run akey-role && python3 tools/scenario_run.py run weapon-pickup && python3 tools/scenario_run.py run hit-test
grep -n "STA     \$33\b\|STA     \$34\b" reference/seed/gunnac.asm      # 拾取/初始化/重置写点
python3 tools/da6502.py rom/gunnac_cn.nes 05222 9222 40                 # 拾取处理 CMP/INC/STA 链
python3 tools/da6502.py rom/gunnac_cn.nes 05202 9202 26                 # 关卡初始化 STA $33/$34/$3B
grep -n "LDA     \$72E,X\|INC     \$72E,X\|LDA     \$16" reference/seed/gunnac.asm | head  # 自动连射 vs 按键
```
