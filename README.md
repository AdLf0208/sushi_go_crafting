# Sushi Go Crafting for Horizon — the cutting board and the fermentation barrel

A clean-room Rust port of **Sushi Go Crafting 1.21-0.6.7**
([Buuz135/SushiGoCrafting](https://github.com/Buuz135/SushiGoCrafting), **MIT**)
for [Horizon](https://github.com/horizon-rust/horizon): Minecraft Java 26.3, one
flat SDK package (mod API **0.2**) built against nightly
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.

**One file ships:** `sushigocrafting.hmod` carries the code (`src/`) *and* the
content (`assets/`, `data/`), so installing it gives the block, the items and the
behaviour together.

## What it implements

### The fermentation barrel (slice 2)

An input slot and an output slot, a **2,000 mB tank** and a **100-tick** bar, with
the jar's two recipes:

| Input | Fluid | Output | Amount |
|---|---|---|---|
| *(empty slot)* | 250 mB `minecraft:milk` | `sushigocrafting:cheese` | 50..249 |
| `#c:crops/soybean` | 250 mB `minecraft:water` | `sushigocrafting:soy_sauce` | 10..49 |

A bucket in hand fills the tank (water, lava or milk; the tank never gives fluid
back to a bucket), a batch drains 250 mB and puts the recipe's `AmountItem` in the
output slot, combining into a stack already holding one. An empty-hand click opens
the barrel's menu — two slots and the bar. Its own recipe is all vanilla (seven
planks, a wooden pressure plate and a furnace), so it is craftable in a bare 26.3.
`/sushigocrafting barrel <x> <y> <z>` reads it and `/sushigocrafting tank <x> <y>
<z> <fluid> <mb>` sets the tank (harness controls).

### The cutting board (slice 1)

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

The rice cooker, roller and cooler box; every crop and the avocado tree; the two
fish; the sushi foods and the roller's UI; the three mob effects; worldgen and
loot modifiers; the guide book; and the client halves (the item drawn on the board
and the knife animation, the barrel's tank gauge and progress arrow).

The **ingredients** (cucumber, avocado, wasabi, fish, soy beans, rice) are not
growable or catchable yet — the crop slice is not done — so give them in creative.
`minecraft:milk` is not a fluid Horizon models, so a pipe cannot fill the barrel's
tank with it; a bucket in hand can, and `cheese` is otherwise unobtainable.

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
cargo horizon-mod fits sushigocrafting.hmod --build 0a875da        # `made for`
cargo horizon-mod fits sushigocrafting.hmod --build 0b14f8a        # `fits` (a different build)
python3 tools/check_tags.py                                         # the knife-tag fix
```

`.horizon/sdk/` is fetched by `cargo horizon-mod` (gitignored) and never packed.
`assets/` and `data/` are regenerated from the ATM10 jar by
`python3 tools/assemble_content.py` (reproducible: the same 85 files, byte for
byte).

## The in-game check

```sh
ports/tools/launcher-game.sh build sushigocrafting   # prints GAME=, CTL=, CONTROL_GAME=, RES=
eval "$(ports/tools/launcher-game.sh env sushigocrafting)"
ports/tools/build-slot.sh run --job e2e-sushigocrafting --port ports/sushigocrafting \
    --step e2e -- bash ports/sushigocrafting/tools/e2e.sh
# SELFTEST=1 inverts the cheese amount assertion and must FAIL
```

A native `.hmod` cannot load on an unmodded game and there is no `--dev-mod`
route for it, so the game must be launcher-built with this package. The launch is
`--offscreen`, not `--headless`: the menu legs need the client's renderer.

## Licence

MIT, read from the jar's `META-INF/neoforge.mods.toml` (licence rank 0); the
assets carried from the jar are the same MIT project's. See [`LICENSE`](LICENSE)
and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md), both inside the package.
Local-only until the maintainer approves publishing (`ARCHITECTURE.md` §8).
