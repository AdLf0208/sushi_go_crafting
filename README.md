# sushigocrafting (Horizon port)

A Horizon port of [sushigocrafting](Buuz135/SushiGoCrafting) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- The block.** `sushigocrafting:cutting_board` on the **`minecraft:oak_planks`**
- template with upstream's `Properties.ofFullCopy(BLOCKS.OAK_TRAPDOOR)` numbers
- (hardness 3.0, blast resistance 3.0, wooden sounds, no occlusion), the
- `subfacing`** state property Titanium's `RotationType.FOUR_WAY` uses (the
- jar's own four-variant blockstate selects on it unchanged), and upstream's two
- shapes — `1,0,3..15,1,13` on north/south, `3,0,1..13,1,15` on east/west — as
- outline and collision.
- Placement rotation.** `state_for_placement` (API 76) sets `subfacing` from

## Not yet ported

- The other four machines**: `rice_cooker` (4 input slots + fuel + an 8,000 mB
- water tank + a 100-tick bar), `fermentation_barrel` (1 in, 1 out, a 2,000 mB
- tank, 100 ticks), `roller` (the weighing/crafting UI) and `cooler_box`.
- Every crop** (`rice`, `cucumber`, `soy`, `sesame`, `wasabi`) and the
- avocado tree**: no content crop random-ticks at this pin
- (`ARCHITECTURE.md` §3), so growth and drops are not ported; the crop blocks
- themselves were not carried either (they would be inert).
- The sources of the ingredients**: `ShrimpEntity`, `TunaEntity`, the

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: Buuz135/SushiGoCrafting. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
