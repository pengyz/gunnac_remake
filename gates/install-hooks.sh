#!/usr/bin/env bash
# gates/install-hooks.sh —— 挂接 git hooks：
#   pre-commit  -> python3 gates/cli.py check      （总闸，任一 FAIL 不提交；python3 不可用时自动回退 python/py）
#   commit-msg  -> python3 gates/cli.py commit-msg （触及 data/naming/ data/status/ data/reference/ reference/ rom/ 必须含 Evidence/Confidence）
#
# 挂接是幂等的；既有非本项目 hook 先备份为 <name>.pre-gunna-backup.<ts>。
# 逻辑真相源是 gates/cli.py，本脚本只负责落盘（无第二套扫描逻辑）。
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
hooks_dir=$(cd "$repo_root" && git rev-parse --git-path hooks)
case "$hooks_dir" in
  /*) ;;
  *) hooks_dir="$repo_root/$hooks_dir" ;;
esac

mkdir -p "$hooks_dir"

for name in pre-commit commit-msg; do
  src="$repo_root/gates/hooks/$name"
  dst="$hooks_dir/$name"
  if [ ! -f "$src" ]; then
    echo "错误: 钩子模板缺失 $src" >&2
    exit 1
  fi
  if [ -e "$dst" ] && ! grep -q "gunnac-remake gates" "$dst"; then
    backup="$dst.pre-gunna-backup.$(date +%Y%m%d%H%M%S)"
    mv "$dst" "$backup"
    echo "已备份既有 hook: $dst -> $backup"
  fi
  install -m 755 "$src" "$dst"
  echo "已挂接: $dst -> gates/hooks/$name"
done

echo "验证:"
echo "  bash gates/install-hooks.sh          # 本脚本（幂等）"
echo "  python gates/cli.py check            # pre-commit 实际执行的总闸（Windows 上 python3 占位别名已由 hook 回退）"
echo "  python gates/cli.py commit-msg <f>   # commit-msg 实际执行的证据门禁"
