#!/bin/sh
# Rebuild console WASM into ui/pkg. Console: --ui-dir ui/pkg
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
(cd "$ROOT" && npm run css)
cargo build --manifest-path "$ROOT/ui/Cargo.toml" \
  --target wasm32-unknown-unknown --release \
  --target-dir "$ROOT/ui/target"
OUT="$ROOT/ui/pkg"
mkdir -p "$OUT"
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "cargo install wasm-bindgen-cli --version 0.2.127" >&2
  exit 1
fi
wasm-bindgen "$ROOT/ui/target/wasm32-unknown-unknown/release/connect_console_ui.wasm" \
  --out-dir "$OUT" --target web --out-name connect_console_ui
echo "connect-console --ui-dir $OUT"
echo "reload the tab after each run of this script"
