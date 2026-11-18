# Sushi Go Crafting — a Horizon port

**Sushi Go Crafting** is a Minecraft mod. A food minecraft mod where you make sushi with a small discovery minigame to get the perfect food and get amazing food effects. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Buuz135/SushiGoCrafting](https://github.com/Buuz135/SushiGoCrafting) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Sushi Go Crafting ([source](https://github.com/Buuz135/SushiGoCrafting)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `sushigocrafting.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What this slice implements

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

## Not yet ported

- **The other four machines**: `rice_cooker` (4 input slots + fuel + an 8,000 mB
- **Every crop** (`rice`, `cucumber`, `soy`, `sesame`, `wasabi`) and the
- **The sources of the ingredients**: `ShrimpEntity`, `TunaEntity`, the
- **The container filter for automation.** `container_accepts` is a **pending**
- **Steady Hands** (`c:tools`-independent): upstream adds `(amplifier + 1) * min`
- **The sushi foods and the `FoodItem`/`Roller` crafting chain** that consume the
- **The cleaver-knife recipe needs `#c:ingots/iron`** from another mod: carried,
- **The client half**: `CuttingBoardRenderer` (the item on the board and the

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `sushigocrafting.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Sushi Go Crafting was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Buuz135/SushiGoCrafting

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run sushigocrafting` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/sushigocrafting/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
