# Sushi Go Crafting for Horizon — slice 1: the cutting board

A clean-room Rust port of **Sushi Go Crafting 1.21-0.6.7**
([Buuz135/SushiGoCrafting](https://github.com/Buuz135/SushiGoCrafting), **MIT**)
for [Horizon](https://github.com/horizon-rust/horizon): Minecraft Java 26.3, one
flat SDK package (mod API **0.2**) built against nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4` / mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.

**One file ships:** `sushigocrafting.hmod` carries the code (`src/`) *and* the
content (`assets/`, `data/`), so installing it gives the block, the items and the
behaviour together.

## What this slice implements

The **cutting board**: one item slot that accepts only a cutting-board
ingredient, a knife in the hand that strokes it, **six strokes per cut**, and the
jar's **six recipes** giving their `AmountItem` with the jar's amount range
(`sushigocrafting:amount`):

| Input | Output | Amount |
|---|---|---|
| `#c:fruits/avocado` | `sushigocrafting:avocado_slices` | 100..499 |
| `minecraft:cod` | `sushigocrafting:imitation_crab` | 50..199 |
| `#c:crops/cucumber` | `sushigocrafting:cucumber_slices` | 50..199 |
| `#c:raw_fishes/salmon` | `sushigocrafting:salmon_fillet` | 500..1999 |
| `#c:raw_fishes/tuna` | `sushigocrafting:tuna_fillet` | 1000..2999 |
| `#c:crops/wasabi_root` | `sushigocrafting:wasabi_paste` | 10..49 |

Plus: the block's placement rotation (`subfacing`, toward the placer), the board's
own crafting recipe, an axe tag, a loot table, a creative tab, and two commands —
`/sushigocrafting board <x> <y> <z>` (the slot, the strokes, the next output) and
`/sushigocrafting clicks <x> <y> <z> <n>` (set the stroke count; a harness
control).

A knife is the **union** of `c:tools/knife` and the jar's own `c:tools/knives`, and
the content declares the cleaver under both names: upstream's code tests the
singular tag while its data ships only the plural one, so its own cleaver cannot
cut on its own board (fixed here — see `notes.md`, "Upstream defects fixed").

## What it does not (see `notes.md` §Gaps)

The rice cooker, fermentation barrel, roller and cooler box; every crop, the
avocado tree and the two fish; the sushi foods and the roller's UI; the three mob
effects; worldgen and loot modifiers; the guide book; and the client renderer
(the item drawn on the board and the knife animation).

The machine's **inputs** (cucumber, avocado, wasabi, fish) are not growable or
catchable in this slice: growth is the measured content-crop gap
(`ARCHITECTURE.md` §3). Give them in creative, or wait for the crop slice.

## Building and checking

```sh
# everything runs inside the campaign's build-slot registry, never a bare cargo
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sushigocrafting-sdk \
    --port ports/sushigocrafting --step test -- \
    bash -c 'cd ports/sushigocrafting && CARGO_INCREMENTAL=0 cargo test --offline'

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sushigocrafting-sdk \
    --port ports/sushigocrafting --step build -- \
    bash -c 'cd ports/sushigocrafting && rm -f Cargo.lock && cargo horizon-mod build'

cd ports/sushigocrafting
cp target/horizon-mod/sushigocrafting.hmod ./sushigocrafting.hmod   # the deliverable
cargo horizon-mod check sushigocrafting.hmod                        # exit 0
cargo horizon-mod fits sushigocrafting.hmod --build nightly         # exit 0
python3 tools/check_tags.py                                         # the knife-tag fix
```

`.horizon/sdk/` is fetched by `cargo horizon-mod` (gitignored) and never packed.
`assets/` and `data/` are regenerated from the ATM10 jar by
`python3 tools/assemble_content.py` (reproducible: the same 61 files, byte for
byte).

## The in-game check

```sh
GAME=<horizon-client built by `horizon-launcher native-build ports/sushigocrafting`> \
    tools/e2e.sh                 # SELFTEST=1 flips the amount assertion and must FAIL
```

A native `.hmod` cannot load on an unmodded game and there is no `--dev-mod`
route for it, so the game must be launcher-built with this package.
`horizon-launcher` is **not installed** in the campaign's current environment, so
no 0.2.0 in-game run is claimed; the 0.1.0 runs in
`../_shared/evidence/sushigocrafting/` stand as the behaviour's evidence for a
`runtime.rs` that did not change (`notes.md`, "Gaps after the migration").

## Licence

MIT, read from the jar's `META-INF/neoforge.mods.toml` (licence rank 0); the
assets carried from the jar are the same MIT project's. See [`LICENSE`](LICENSE)
and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md), both inside the package.
Local-only until the maintainer approves publishing (`ARCHITECTURE.md` §8).
