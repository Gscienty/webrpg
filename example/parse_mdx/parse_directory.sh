#!/usr/bin/env bash

set -u

if [[ $# -ne 1 || ! -d $1 ]]; then
    echo "usage: $0 <directory>" >&2
    exit 2
fi

script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
parser="$script_dir/target/debug/webrpg-parse-mdx-example"
log_file=$(mktemp "${TMPDIR:-/tmp}/parse-mdx.XXXXXX")
trap 'rm -f "$log_file"' EXIT

cargo build --quiet --manifest-path "$script_dir/Cargo.toml" || exit 1

total=0
failed=0
while IFS= read -r -d '' file; do
    total=$((total + 1))
    printf '[%d] %s\n' "$total" "$file"

    if "$parser" "$file" >/dev/null 2>"$log_file"; then
        continue
    fi

    failed=$((failed + 1))
    printf '\n解析失败\n文件: %s\n日志:\n' "$file" >&2
    cat "$log_file" >&2
    printf '\n按回车继续，Ctrl-C 退出...' >&2
    IFS= read -r _ </dev/tty || exit 1
    printf '\n\n' >&2
done < <(find "$1" -type f -iname '*.mdx' -print0)

printf '完成: 总计 %d，成功 %d，失败 %d\n' "$total" "$((total - failed))" "$failed"
