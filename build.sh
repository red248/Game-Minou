#!/usr/bin/env bash
# Builds the web app into dist/ (needs: rustup target wasm32-unknown-unknown, wasm-bindgen-cli 0.2.100)
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
rm -rf dist && mkdir -p dist
wasm-bindgen --target web --no-typescript --out-dir dist --out-name minou \
  target/wasm32-unknown-unknown/release/minou.wasm
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int -o dist/minou_bg.wasm dist/minou_bg.wasm
fi
VERSION=$(cat dist/minou_bg.wasm web/style.css | sha1sum | cut -c1-10)
for f in web/*; do
  name=$(basename "$f")
  case "$name" in
    *.html|*.js|*.webmanifest) sed "s/__VERSION__/$VERSION/g" "$f" > "dist/$name" ;;
    *) cp "$f" "dist/$name" ;;
  esac
done
echo "Built dist/ (version $VERSION)"
