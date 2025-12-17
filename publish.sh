#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Publishing top-level crates..."

for crate_dir in module/iron_cli module/iron_control_api module/iron_runtime; do
  echo ""
  echo "=== Publishing $crate_dir ==="
  cd "$crate_dir"
  will .publish dry:0
  cd - > /dev/null
done

echo ""
echo "✅ All 3 top-level crates published"
