#!/usr/bin/env bash
# The final ladder in one lease: fmt, clippy, the unit tests, both packages, and
# the bundle check. Run inside a build slot:
#   BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sgc-final \
#       --port ports/sushigocrafting --step test -- env CARGO_INCREMENTAL=0 tools/final_ladder.sh
set -euo pipefail
PORT=$(cd "$(dirname "$0")/.." && pwd)
cd "$PORT"
export CARGO_INCREMENTAL=0

echo "== fmt (the crate's lib source) =="
cargo fmt --check
echo "== fmt (the injected glue files, which are not in the crate graph) =="
rustfmt --check --edition 2024 mixins/sushigocrafting/*.rs
echo "== clippy --all-targets -D warnings =="
cargo clippy --all-targets --locked --offline -- -D warnings
echo "== cargo test (the standalone runtime) =="
cargo test --locked --offline 2>&1 | tail -20
echo "== packages =="
tools/package.sh
echo "== bundle check =="
cargo horizon-mod check sushigocrafting.hnmod --kind native-mod
