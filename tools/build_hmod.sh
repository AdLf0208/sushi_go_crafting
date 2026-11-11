#!/usr/bin/env bash
# Build the content half and put it where both the staged check and the bundle
# need it:
#   content/sushigocrafting.hmod                              the delivered content mod
#   mixins/sushigocrafting/data/sushigocrafting-content.hmod   the same bytes, which the
#                                                              mixins planner stages for
#                                                              `native_data!`
# Run inside a build slot:
#   ports/tools/build-slot.sh run --job sgc-hmod --port ports/sushigocrafting \
#       --step package -- env CARGO_INCREMENTAL=0 tools/build_hmod.sh
set -euo pipefail

PORT=$(cd "$(dirname "$0")/.." && pwd)
HORIZON=${HORIZON:-${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}/ports/_shared/pins/horizon-1d63e0a}
LOGS=$PORT/target/ladder
mkdir -p "$LOGS" "$PORT/mixins/sushigocrafting/data" "$PORT/target/carry-tools"
cd "$PORT"

CARGO_TARGET_DIR=$PORT/target/carry-tools
export CARGO_TARGET_DIR
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod build) \
    2>&1 | tee "$LOGS/package-hmod.log"
cp content/target/horizon-mod/sushigocrafting.hmod content/sushigocrafting.hmod
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod check sushigocrafting.hmod) \
    2>&1 | tee -a "$LOGS/package-hmod.log"
cp content/sushigocrafting.hmod mixins/sushigocrafting/data/sushigocrafting-content.hmod

size=$(stat -c %s content/sushigocrafting.hmod)
sha=$(sha256sum content/sushigocrafting.hmod | cut -d' ' -f1)
entries=$(python3 -c "import zipfile,sys; print(len(zipfile.ZipFile(sys.argv[1]).namelist()))" content/sushigocrafting.hmod)
echo "content/sushigocrafting.hmod size=$size sha256=$sha entries=$entries"
