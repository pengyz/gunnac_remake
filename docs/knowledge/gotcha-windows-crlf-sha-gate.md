---
type: gotcha
title: Windows 检出/文本写入的 CRLF 会让 sha256 门禁误报——需 LF 双修（检出侧+工具写入侧）
date: 2026-10-18
status: confirmed
---

## 现象

在 Windows 上检出本仓库后，`python gates/cli.py status` 报三项 FAIL：
`rom-integrity`（"基线文件被手工改动"）、`seed-integrity`（seed sha 不符）、
`baseline-integrity`（baseline sha 链断块）。但 `git status` 干净、`git log` 无异常提交，
`git cat-file blob` 取出的仓库内 blob 与 `data/status/baselines/SHA256SUMS` 锚点**逐字一致**。

第二面（同一根因家族）：门禁/工具自身用 `Path.write_text(...)` / `open(..., "w")` 写受跟踪文件时，
Windows 文本模式默认 `newline=None` 把 `"\n"` 译为 CRLF，导致：
- `gates/cli.py` 的 `save_baseline` / `sync_state_anchors` / `append_history` 在 Windows 执行
  `baseline update` 会**自毁 sha 链**（历史 `baseline_sha256` 按 LF payload 算，磁盘却是 CRLF）；
- `tools/diff_visual.py` 每次把 `oracle/screens/whitelist-application.md` 写成 CRLF，
  使每次 `gates status/check` 之后工作区变脏（归一后内容等值、字节不等）。

## 为什么

- `core.autocrlf=true`（本机来自系统级 `C:/Program Files/Git/etc/gitconfig`）的转换只发生在
  **检出/提交**环节，不改变"磁盘字节 → sha256"的事实；门禁按磁盘原始字节校验，故误报。
- Python 文本模式写入按 `os.linesep` 翻译（Windows=CRLF）；Linux/CI 上恰好都是 LF 而不可见。
- 两个坑都会让**合法操作被判为篡改**，容易被误判为"基线被手工改动"，从而走进错误的取证方向。

## 何时使用

- 在 Windows 上跑/改门禁、基线、派生产物之前先看本条。
- 看到"基线文件被手工改动"而 `git status` 干净时：**先查 CRLF，再怀疑篡改**
  （判据：`git cat-file blob HEAD:<path>` 的 sha 与锚点是否一致）。
- 新增任何会写受跟踪文件的工具时：显式 `newline="\n"`（或写 bytes）。
- 需要强制把工作区刷回 LF 时：`git rm --cached -r . && git reset --hard`
  （实测 `git checkout -- .`、`git checkout-index -f -a` 会因 stat 缓存跳过、不转换）。

## 证据指针

- commit `04559b1`（2026-10-18）：新增 `.gitattributes`（`* text=auto eol=lf` + 二进制显式 `binary`），
  全仓库 LF 归一；取证：`git cat-file blob` 四锚点（rom/baseline.json/history/addr-bridge）与
  `SHA256SUMS` 全等、仓库内 0 个文本 blob 含 CRLF ⇒ 零篡改、零内容变更。
- commit `5bb56c1`（2026-10-18）：`gates/cli.py` 3 处 + `tools/diff_visual.py` 1 处写入显式
  `newline="\n"`；自证 `gates/cli.py selftest → 37/37`。
- 本机实测：`Path.write_text(默认) → CRLF`、`newline="\n" → LF`；git `2.49.0.windows.1`、Python 3.12.1。
- 修复后基线：`gates/cli.py status → PASS=11 FAIL=0 SKIP=1`；`check → CHECK: PASS`；
  磁盘文本 CRLF 文件数 = 0（含视觉门禁重写派生文档后）。
