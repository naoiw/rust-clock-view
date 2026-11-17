#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
wasm-pack build --release --target web --out-dir dist/pkg
mkdir -p dist
cp src/styles.css dist/styles.css
cp src/main.js dist/app.js
