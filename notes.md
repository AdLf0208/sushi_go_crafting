# Sushi Go Crafting — port notes (slice 1: the cutting board; slice 2: the fermentation barrel)

Ported from **Sushi Go Crafting 1.21-0.6.7** (`Buuz135/SushiGoCrafting`, **MIT**)
for Minecraft Java 1.21.1 / NeoForge. Jar
`ports/_shared/licence-cache/sushigocrafting-1.21-0.6.7.jar` (`unzip -l`: 640
entries), licence read from its `META-INF/neoforge.mods.toml`. Behaviour read
from the upstream repository, branch `1.21`, commit
**`c3b296bf81b4f2a712ea1a3c58cbef0f86746f55`** (2026-10-27, the jar's build day),
cloned shallow to `ports/_shared/upstream/sushigocrafting` and deleted once
[`FINDINGS.md`](FINDINGS.md) was written. The Rust is a clean-room rewrite: no
Java was copied and no class was decompiled.

Pins: slice 1 was built at **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, migrated to the
SDK pin **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (0.2.0/0.2.1), and slice 2 is built at the
campaign pin **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (nightly 2026-11-20, `package_format` 6,
`content_api` "0.2"); mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** throughout.

Local-only until the maintainer approves publishing (`ARCHITECTURE.md` §8).
**Licence rank 0** (MIT) — the licence was read from the jar, not from a
catalogue.

## Class + tier and the approach (from the pin's `docs/modpacks.md`)

- **Class `content`, tier `machines`** (`ports/ledger.csv` row `sushigocrafting`;
  provenance `ports/_shared/source-map/atm10-sources.csv`), core, rank 0.
- **Approach:** a machine-tier shape — a block entity with one item slot, a
  container the engine treats as a machine's (`ModContainer`), a placement
  state, an interaction that advances the machine's own state, and the carried
  block/items/data. The board has **no menu and no tick** upstream: its whole
  state is one slot and a knife-stroke count, and its output goes to the player
  (`CuttingBoardTile.onActivated`), so this slice is a block entity + container
  machine rather than a GUI machine.
- **Carry-over vs agent port:** the blockstate, block model, texture, the eleven
  item models/textures, the lang strings, the tags, the crafting recipe and the
  loot table are carried from the jar (spellings and decisions in
  [`CONTENT.md`](CONTENT.md)); the recipe table, the stroke machine, the amount
  roll, the block entity, the container, the interaction, the placement rotation
  and the readout are the agent port.

## Why this machine, and why not crop growth

The brief fixes the shape: **one machine loop that closes without crop growth**.
The board is the smallest such loop in the jar — item in, knife strokes, item out
— with six recipes of its own and no fluid, fuel or growth dependency. The rice
cooker and the fermentation barrel are the other candidates (`FINDINGS.md`), and
both need a fluid tank plus `AmountItem` outputs; they are the next slices.

The inputs the board takes (`cucumber`, `avocado`, `wasabi_root`, `raw_tuna`,
`minecraft:salmon`, `minecraft:cod`) come from the mod's **crops**, its **avocado
tree** and its **fish** upstream. Those are out of this slice: at this pin a
content crop on the `wheat` template **does not random-tick** and a
`random_tick` (API 33) injection never fires for a content block — the measured
gap in `ARCHITECTURE.md` §3 that made `farmersdelight` `blocked(crop-growth)`,
and it is cited here rather than re-derived. The machine's loop itself has no
growth in it: creative play and `ctl give` provide the ingredients, and the e2e
does exactly that.

## What the slice does

- **The block.** `sushigocrafting:cutting_board` on the **`minecraft:oak_planks`**
  template with upstream's `Properties.ofFullCopy(BLOCKS.OAK_TRAPDOOR)` numbers
  (hardness 3.0, blast resistance 3.0, wooden sounds, no occlusion), the
  **`subfacing`** state property Titanium's `RotationType.FOUR_WAY` uses (the
  jar's own four-variant blockstate selects on it unchanged), and upstream's two
  shapes — `1,0,3..15,1,13` on north/south, `3,0,1..13,1,15` on east/west — as
  outline and collision.
- **Placement rotation.** `state_for_placement` (API 76) sets `subfacing` from
  the placer's yaw; the mapping is `runtime::facing_for_yaw`, unit tested. The
  e2e shows the placed block is `cutting_board[subfacing=west]` for a placer
  standing west of it. A headless run cannot check what the model *looks* like
  (gap 8).
- **The block entity** (API 15) keeps the one slot (`Items`) and the stroke count
  (`clicks` in its compound, upstream's `@Save int click`).
- **The container** (API 16): one slot, every face, no menu — upstream's
  `InventoryComponent("input", 0, 0, 1)` with a filter and no `openGui`.
- **The interaction** (API 21, `right_click_block`), upstream's `onActivated`
  order: a knife on a loaded board strokes it (`++click`); an ingredient on an
  empty board fills the slot; an empty hand while sneaking takes the stack back;
  anything else falls through. **Six strokes cut**: the sixth gives the recipe's
  `AmountItem` (one stack, `sushigocrafting:amount` between the jar's `min` and
  `max-1`) to the player and takes one from the slot.
- **The recipe table** (`runtime::RECIPES`) is the jar's six
  `recipe/cutting_board/*.json` rows with the `FoodIngredient` name→item mapping
  and the `SushiContent.Items` amount bounds; a unit test pins all six against
  `FINDINGS.md`'s transcription of the files.
- **Harness commands** (API 8), `/sushigocrafting board <x> <y> <z>` and
  `/sushigocrafting clicks <x> <y> <z> <n>`: upstream has none. The readout is
  `runtime::summary` (the slot, the strokes, the next output and its range); the
  setter exists because `ctl interact` presses and releases across two ticks and
  can be heard twice (a harness fact, below), so a leg can stand at five strokes
  exactly (precedent: `/industrialforegoing progress`, `/powah set`).
- **One file ships.** The built `content/sushigocrafting.hmod` rides inside the
  `.hnmod` at `mixins/sushigocrafting/data/` and a `content_root` injection
  writes it out and appends it to the dev list (the AE2 pattern).

## Hooks (all stamped at the pin; none pending)

| Hook (API) | Helper | What it does |
|---|---|---|
| `component_types` (65) | `component_types.rs` | registers `sushigocrafting:amount` as a component type |
| `block_entity` (15) | `block_entity.rs` | the board's block entity |
| `block_entity_container` (16) | `block_entity_container.rs` | its one slot, every face, no menu |
| `state_for_placement` (76) | `state_for_placement.rs` | `subfacing` from the placer's yaw |
| `right_click_block` (21) | `right_click_block.rs` | insert / stroke / cut / hand back |
| `register_commands` (8) | `register_commands.rs` | `/sushigocrafting board\|clicks` |
| `perform_command` (8) | `perform_command.rs` | the readout and the stroke setter |
| `content_root` (HEAD) | `embedded_content.rs` | writes the embedded `.hmod` out and loads it |

Not used, deliberately: `container_accepts` (still **pending** at this pin, so a
hopper's wrong item is not refused — gap 4) and `block_entity_update_tag` (no
client renderer in this slice).

## Behaviour read from upstream, and where this port differs

| Area | Upstream | This port |
|---|---|---|
| slot | 1, filter = "some cutting-board recipe's input matches" | same for the hand and the readout; hoppers reach it unchecked (gap 4) |
| insert | the whole held stack, `setCount(0)` (lossy when the slot is partly full) | what fits under 64; the rest stays in the hand |
| knife | `c:tools/knife` (singular) | the union of `c:tools/knife` and the jar's own `c:tools/knives` |
| strokes | `++click; if (click > 5)`, reset to 0 | the same, unit tested |
| cut | every matching recipe's item, `shrink(1)` | one item (the inputs are disjoint), `shrink(1)` |
| amount | `min(maxCombine, extra + rand.nextInt(max-min) + min)` with Steady Hands `extra` | the same without `extra` (no way to read a player's effects at this pin — gap 5) and through the registered component |
| output | `ItemHandlerHelper.giveItemToPlayer`, dropped nowhere | `native_give_stack`, and what does not fit is dropped at the board |
| sneak-hand-back | empty hand + sneak → whole stack to the player | the same |
| menu | none (no GUI in the jar) | none |
| tick | none (the TE's tick does nothing) | none |
| rotation | `FOUR_WAY` toward the placer | the same, yaw mapping unit tested; the *look* is unverified (gap 8) |
| renderer | draws the item and a knife animation | not drawn (gap 8) |
| comparator | none upstream | none |

## Deliberate deviations (and why)

1. **The knife tag is the union of `c:tools/knife` and `c:tools/knives`, and the
   cleaver is declared under both names.** Upstream's code tests the singular tag
   while its own data ships only the plural one holding `cleaver_knife`, so in a
   pack without Farmer's Delight the mod's own knife cannot cut on its own board.
   That is an upstream **defect**, fixed here rather than reproduced: the runtime
   keeps accepting either tag (other mods' knives work) and the content declares
   both (`## Upstream defects fixed`). The e2e proves the jar's knife cuts and a
   stick does not.
2. **Insertion is not lossy.** Upstream inserts the whole held stack and zeroes
   it, which deletes the overflow when the slot is partly full; this port inserts
   what fits and leaves the rest in the hand.
3. **The amount component is set through `native_set_component` (API 65) rather
   than written as raw NBT.** An unregistered component is kept as an unknown
   `components` entry and does not survive to the client (measured: the e2e's
   `ctl inventory` showed no components for the stack until the type was
   registered).
4. **`/sushigocrafting clicks` exists.** A player can set the stroke count with
   it; it is a debug control like `/setblock`, not a behaviour the mod adds. The
   e2e needs it because `ctl interact` can fire the use twice per command.
5. **One item per cut.** Upstream loops over *all* matching recipes; the six
   inputs are disjoint (`item` vs four distinct tags), so one cut yields one
   item in both.

## Upstream defects fixed

### The cutting board's knife tag (fixed 2026-11-12)

**The defect, proved from the artefact.** Upstream's board tests the **singular**
convention tag while its own data declares only the **plural** one:

- `com.buuz135.sushigocrafting.tile.machinery.CuttingBoardTile.class` holds the
  constant `tools/knife` — read as a literal string from the jar, no decompiling:
  `unzip -p ports/_shared/licence-cache/sushigocrafting-1.21-0.6.7.jar
  'com/buuz135/sushigocrafting/tile/machinery/CuttingBoardTile.class' | strings`
  prints `KNIFE` and `tools/knife` — so `onActivated` asks whether the held stack
  is in `c:tools/knife`;
- the jar's only shipped knife tag is `data/c/tags/item/tools/knives.json` (57 B),
  `{"values": ["sushigocrafting:cleaver_knife"]}`. There is **no**
  `data/c/tags/item/tools/knife.json` in the jar, and no class adds the cleaver to
  the singular tag at runtime. Re-checked in the pack build `1.21-0.6.7` and the
  newest release `26.1-0.6.7` (both the same).

So in a pack without Farmer's Delight — which supplies `c:tools/knife` — the mod's
own cleaver cannot cut on its own board.

**The fix (minimal, and compatible).** The runtime tests the **union**
(`runtime::KNIFE_TAGS = ["c:tools/knife", "c:tools/knives"]`, unit tested with both
spellings), and the content now **declares the cleaver under both names**:
`data/c/tags/item/tools/knife.json` and `data/c/tags/item/tools/knives.json`, both
written by `tools/assemble_content.py` (`TAGS`), so a content rebuild cannot drop
either. Nothing upstream-valid is lost: another mod's knife matches whichever of
the two tags it declares.

**What a user notices.** The mod's own cleaver cuts on its own board with no other
mod installed, and either tag spelling in another mod's data works. The board's
other rules are unchanged (a stick still strokes nothing).

**Proof (fail-capable).** `tools/check_tags.py` opens the delivered
`content/sushigocrafting.hmod` **and** the `.hmod` embedded in
`sushigocrafting.hnmod`, asserts that both tag files exist and each lists
`sushigocrafting:cleaver_knife`, and exits 1 otherwise. It runs inside
`tools/package.sh` and at the top of every `tools/e2e.sh` leg, so a package that
reproduced the defect fails before a world opens. Its `--selftest` puts the defect
back in memory (the singular file removed; the file present but without the
cleaver) and proves both shapes are rejected. The pre-fix package is the red
evidence: `ports/_shared/evidence/sushigocrafting/tags-check-red.log` — both
artifacts lacked `data/c/tags/item/tools/knife.json` while carrying the plural one.

**Worth an upstream report?** Yes — it is a one-line data fix upstream (ship
`data/c/tags/item/tools/knife.json`, or test `c:tools/knives` in
`CuttingBoardTile`). Not filed: this campaign opens upstream issues only when the
orchestrator asks.

## Gaps (next slices)

1. **The other four machines**: `rice_cooker` (4 input slots + fuel + an 8,000 mB
   water tank + a 100-tick bar), `fermentation_barrel` (1 in, 1 out, a 2,000 mB
   tank, 100 ticks), `roller` (the weighing/crafting UI) and `cooler_box`.
2. **Every crop** (`rice`, `cucumber`, `soy`, `sesame`, `wasabi`) and the
   **avocado tree**: no content crop random-ticks at this pin
   (`ARCHITECTURE.md` §3), so growth and drops are not ported; the crop blocks
   themselves were not carried either (they would be inert).
3. **The sources of the ingredients**: `ShrimpEntity`, `TunaEntity`, the
   `avocado_leaves_ripe` drop, the seed loot modifiers (`grass_seeds`,
   `item_amount`) and the worldgen/biome modifiers.
4. **The container filter for automation.** `container_accepts` is a **pending**
   hook at this pin, so a hopper or `native_insert_item` can put any item in the
   board's slot; the machine still refuses to cut a non-recipe (the e2e shows
   dirt stays `next=none`). One line to fix when the hook is stamped.
5. **Steady Hands** (`c:tools`-independent): upstream adds `(amplifier + 1) * min`
   to a cut when the player has the effect; there is no stamped way to read a
   player's active effects, so the bonus is not applied. The three mob effects
   (`acquired_taste`, `small_bites`, `steady_hands`) are not registered at all.
6. **The sushi foods and the `FoodItem`/`Roller` crafting chain** that consume the
   AmountItems; the `combine_amount` recipe; `nori_sheets`, `tobiko`, `cheese`,
   `soy_sauce`, `shrimp`, `cooked_rice` and the fish buckets, whose items this
   slice does not declare.
7. **The cleaver-knife recipe needs `#c:ingots/iron`** from another mod: carried,
   but it cannot match in a bare 26.3 (recorded, not silently dropped).
8. **The client half**: `CuttingBoardRenderer` (the item on the board and the
   knife animation), the model's visual orientation for each `subfacing`, and any
   screen (upstream has none).
9. **The Patchouli guide book** and the 65 `patchouli_books` files.
10. **`combine_amount`, KubeJS scripts, CraftTweaker/JEI/Diet compat** — data and
    integration for other slices.

## Harness facts learned here (for the next slice)

- **`ctl interact` can be heard twice.** The command presses the right button and
  releases it across two ticks, so one `ctl interact` on the board counted two
  strokes (`clicks=2`). The e2e asserts `1..2` for a single stroke and sets the
  exact count with the port's own command before the sixth-stroke leg.
- **`ctl container` reaches a mod container.** Inserting dirt through it worked
  and the readout showed `input=minecraft:dirt`: the container path is real, and
  with `container_accepts` pending it takes the wrong item (gap 4).
- **`ctl block` returns `state`**, so the placed `subfacing` is visible as
  `sushigocrafting:cutting_board[subfacing=west]`.
- **`ctl inventory` prints a stack's components** as SNBT — the only way to see
  the amount (`{"sushigocrafting:amount":149}`), and the check that the
  registration in deviation 3 is what makes it appear.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Verification

| Step | Result | Evidence |
|---|---|---|
| runtime unit tests (red) | 1 passed, 8 failed on `todo!()` | `ports/_shared/evidence/sushigocrafting/test-red.log` |
| runtime unit tests (green) | **10 passed**, 0 failed | `ports/_shared/evidence/sushigocrafting/test-green.log` |
| `cargo fmt --check` + `rustfmt --check` (glue) / `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `ports/_shared/evidence/sushigocrafting/ladder.log` |
| `tools/horizon.py stage` | plan accepted (`stage_changed: true`) | `ladder-stage-check-test.log` |
| `tools/horizon.py check` | `cargo_exit: 0`, 0 warnings | `ladder-stage-check-test.log` |
| `tools/horizon.py test` | **10 passed** as `game::sushigocrafting::tests` (`1240 filtered out`) | `embedded-tests.log` |
| `tools/horizon.py build` | `cargo_exit: 0` (fast profile, every e2e leg) | e2e logs |
| structural: both knife tags in the `.hmod` and the bundle | `PASS` — `knife.json` and `knives.json`, each `["sushigocrafting:cleaver_knife"]`, in both artifacts; `--selftest` rejects both defect shapes | `tags-check-red.log` (red, pre-fix), `tags-check-selftest.log`, every `e2e-*.log` |
| e2e, dev content | `PASS` + `EXIT:0` | `e2e-default.log` |
| e2e, embedded (no `--dev-mod`) + control | `PASS` + `EXIT:0`; control: ids unknown, embedded file **36,600 B** byte-identical | `e2e-embedded.log` |
| e2e, `SELFTEST=1` | `FAIL` + `EXIT:1` (amount 192 vs an impossible 200..199) | `e2e-selftest.log` |
| `check-e2e-logs.py sushigocrafting --strict` | 3/3 logs, `0` missing exits | command output in `VALIDATION.md` |
| packages | `content/sushigocrafting.hmod` **36,600 B** `417fe7b2…d9a134b` 64 entries; `sushigocrafting.hnmod` **42,467 B** `71de4a42…e592b88` 15 entries; two runs identical | `package-run1.log`, `package-run2.log` |

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`: the registry
closed every `.hnmod` submission on 2026-11-18 with one reason, and an `.hnmod` no longer loads.
This port's former submission, **PR #818** (`sushigocrafting` 0.1.0), was closed by that purge.
**0.2.0** is the same behaviour republished as one flat SDK `.hmod`; the changelog text is below.

Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, `package_format` 6,
`content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. The pinned SDK is cached
in-tree at `../_shared/sdk/0b14f8a/` and copied into `.horizon/sdk/` (gitignored) so the ladder runs
offline. The reference recipes were `ports/imfast/notes.md` (the 12-step migration) and
`ports/naturesaura/notes.md` (a block entity, a container, a click Part and commands — this port's
shape).

**Done in this order:** the retired paths were moved into
`ports/_shared/evidence/sushigocrafting/retired-hnmod-0.1.0/` **before** the first
`cargo horizon-mod build` (the tool refuses a root holding retired package paths), `src/runtime.rs`
was copied with `cp` and proved with `cmp` + `sha256sum`, `cargo test` ran on the new crate **before
any glue existed** (**10 passed**), and only then were `src/board.rs` and the crate root written.

### What moved, and where the retired tree is

| 0.1.0 path | Now | Evidence |
|---|---|---|
| `mixins/sushigocrafting/runtime.rs` | `src/runtime.rs`, **copied with `cp`, `cmp`-checked** | `retired-hnmod-0.1.0/mixins/sushigocrafting/runtime.rs` |
| `mixins/sushigocrafting/support.rs` + 8 hook helpers | `src/board.rs` (glue) + `src/lib.rs` (the one `Hooks`); the helper files are gone | `retired-hnmod-0.1.0/mixins/sushigocrafting/` |
| `mixins/sushigocrafting/horizon-mixin.toml` (2 `[[module]]`, 8 `[[injection]]`) | no plan: typed `Hooks`/Part methods and a `Registrar` declaration | `retired-hnmod-0.1.0/mixins/` |
| `mixins/sushigocrafting/data/sushigocrafting-content.hmod` + `embedded_content.rs` (the AE2 write-out) | gone: the package carries `assets/` and `data/` itself | `retired-hnmod-0.1.0/mixins/` |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, `sushigocrafting.hmod`) | the port root: `assets/`, `data/`, and the root `horizon-mod.toml` | `retired-hnmod-0.1.0/content/` |
| `sushigocrafting.hnmod` (15 entries, 42,467 B) | `sushigocrafting.hmod` (70 entries, 61,024 B; native **and** content) | `retired-hnmod-0.1.0/sushigocrafting.hnmod` |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in `horizon-mod.toml`; the pinned commit decides the toolchain | `retired-hnmod-0.1.0/` |
| `tools/horizon.py`, `tools/package.py`, `tools/package.sh`, `tools/test_stage.py`, `tools/test_template.py`, `tools/build_hmod.sh`, `tools/final_ladder.sh`, `tools/notice.py`, `tools/audit/` | deleted: staging, `.hnmod` packaging and the plugin-injection audit belong to the retired format | `retired-hnmod-0.1.0/tools/` |
| `PLAN.md`, the port's old `AGENTS.md` | retired with the format (they tell the reader to open `compatibility.json` and run `tools/horizon.py`) | `retired-hnmod-0.1.0/PLAN.md`, `AGENTS.md.retired` |
| `Cargo.lock` | not kept: it is packed and changes the artifact (trap 5) | not kept |

**Kept and updated:** `tools/e2e.sh` (now needs a launcher-built `GAME=`), `tools/check_tags.py`
(reads the delivered `.hmod`), `tools/assemble_content.py` (writes the package root), `tools/parse_info.py`,
`tools/amount_of.py`, `notes.md`, `README.md`, `CONTENT.md`, `FINDINGS.md`, `VALIDATION.md`, `LICENSE`,
`LICENSE-NOTICE.md`.

### File map (0.2.0)

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` pinned to `0b14f8a`/`e59da90` |
| `Cargo.toml` | one crate, edition 2024; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `component_types`, `register_commands`, `run_mod_command` |
| `src/board.rs` | the glue: the block Part (click + placement state), the block entity type and container declarations, the saved stroke count, the readout and the command bodies. 3 unit tests |
| `src/runtime.rs` | the pure truth (the six recipes, the stroke machine, the amount roll, `fits`, `facing_for_yaw`, `summary`), **byte-identical**, 10 tests |
| `assets/` (37 files), `data/` (24 files) | the carried content, moved from `content/`; byte-identical (see the carry log) |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | inside the package |
| `tools/e2e.sh` | the in-game scenario, now `GAME=`-driven |
| `tools/check_tags.py` | the fail-capable proof of the knife-tag fix, on the delivered `.hmod` |
| `sushigocrafting.hmod` | the deliverable: 61,024 B, sha256 `44a2a0a9…8d10e`, 70 entries |

### Helper → hook table (every retired injection has a home or a stated gap)

| 0.1.0 injection (hook, API) | 0.2.0 |
|---|---|
| `embedded-content` (`content_root`, HEAD, cancellable) | **gone** — one flat `.hmod` carries the content; there is nothing to write out |
| `component-types` (`component_types` 65) | `Hooks::component_types` → `event.types.push("sushigocrafting:amount")` |
| `block-entity` (15) | the block entity **type** declared in `Hooks::register` (`block_entity_type("cutting_board").blocks(&["cutting_board"])`) — the declaration *is* the retired hook: one block entity, empty compound |
| `block-entity-container` (16) | `Registrar::block_container(&block, ModContainer { 1 slot, all six faces, no menu })` — a declaration, not a hook |
| `state-for-placement` (76) | `BlockPart::state_for_placement` on `BoardPart` (`level.state_with_property(state, "subfacing", facing_for_yaw(yaw))`) |
| `right-click-block` (21, `*canceled = true`) | `BlockPart::use_item_on` + `use_without_item` on `BoardPart`, answering `Some(InteractionResult::Success)` where the hook cancelled |
| `register-commands` (8) | `Hooks::register_commands` → two `Command`s |
| `perform-command` (8, string words) | `Hooks::run_mod_command` on typed arguments (`<pos:block_pos>`, `<clicks:integer>`) |
| — | **no gap**: all eight injections mapped |

`uses.json` in the package lists what the build recorded: **335 surface entries** — 25 hook entries
over 10 hooks (`block_entity`, `blocks`, `command`, `component_types`, `ingredient`,
`register_commands`, `run_mod_command`, `state_for_placement`, `use_item_on`, `use_without_item`),
35 methods, 5 Part kinds, 74 handles (`PlayerRef::give_stack`, `BlockEntityRef::set_item_slot`,
`ItemPlace::extract`, `LevelRef::with_random`, …), 26 traits, 167 types, the two macros
(`horizon_mod`, `horizon_part`) and the `registries` capability.

### The ladder (all inside one build slot, `CARGO_INCREMENTAL=0`, no `--target-dir`)

`build-slot.sh` still speaks the retired `tools/horizon.py`, so it is used as the slot guard, not as
the driver: `BUILD_LOCAL=1 build-slot.sh run --step test|lint|fmt|build -- bash -c 'cd ports/<id> && …'`.

| Step | Command | Result |
|---|---|---|
| runtime tests, **before any glue** | `cargo test --offline` | **10 passed**, 0 failed — `ports/_shared/evidence/sushigocrafting/sdk-test-runtime-before-glue.log` |
| tests | `cargo test --offline` | **13 passed**, 0 failed (10 `runtime::`, 3 `board::`) — `ports/_shared/evidence/sushigocrafting/sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** — `ports/_shared/evidence/sushigocrafting/sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`: that checks the fetched SDK) | exit 0, clean — `ports/_shared/evidence/sushigocrafting/sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk` / `Built ./target/horizon-mod/sushigocrafting.hmod` — `ports/_shared/evidence/sushigocrafting/sdk-build.log` |
| registry check | `cargo horizon-mod check sushigocrafting.hmod` | see below, **exit 0** — `ports/_shared/evidence/sushigocrafting/sdk-check.log` |
| fits the pinned build | `cargo horizon-mod fits sushigocrafting.hmod --build nightly` | `Sushi Go Crafting:` / `0b14f8a  made for`, **exit 0** — `ports/_shared/evidence/sushigocrafting/sdk-fits.log` |
| same package twice | `cargo horizon-mod build` again | 61,024 B and `44a2a0a9…8d10e` both times — `ports/_shared/evidence/sushigocrafting/sdk-reproducible.txt`, `sdk-build-second.log` |
| `runtime.rs` untouched | `unzip -p sushigocrafting.hmod src/runtime.rs \| sha256sum` | `ef62dee6…a4197` = the retired file and `src/runtime.rs`, all three — `ports/_shared/evidence/sushigocrafting/sdk-runtime-sha256.txt` |
| content carry | `diff -r assets\|data …/content/{assets,data}` + a generator re-run | no differences; the 61 files reproduce byte for byte — `ports/_shared/evidence/sushigocrafting/sdk-content-carry.log` |
| knife tags | `python3 tools/check_tags.py` and `--selftest` | `PASS` in the package; both defect shapes rejected — `ports/_shared/evidence/sushigocrafting/sdk-tags-check.log` |

`check` output:

```
sushigocrafting 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  61024 bytes, SHA-256 44a2a0a9795ed8887acecfe460859342f0ef179bd9e8de95a3e9b8aae188d10e, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

`fits` output:

```
Sushi Go Crafting:
0b14f8a  made for
```

The archive's 70 entries are `horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`,
`README.md`, `src/{lib,board,runtime}.rs`, `assets/` (37), `data/` (24) and `uses.json`. **No
`Cargo.lock`.** (`notes.md`, `FINDINGS.md`, `CONTENT.md` and `VALIDATION.md` stay outside the
package; the packer takes root documents by name, and `README.md` is the one that ships.)

### Traps this migration hit (new to this port)

1. **A half-failed move leaves a nested tree the packer refuses.** The first attempt copied
   `content/assets` and `content/data` and *then* died on a relative-path `mv`; re-running the script
   copied them *into* the existing folders, so the package root held `assets/assets/sushigocrafting/…`
   and the build answered `error: assets/assets/sushigocrafting/blockstates/cutting_board.json: a mod
   adds files in its own namespaces`. Check the target is absent (or use `cp -a content/assets/. assets/`)
   and re-assert `diff -r` against the retired copy afterwards.
2. **`build-slot.sh` forwards `test`/`build` to the remote runner, and the far side already `cd`s
   into the synced port.** The brief's `cd ports/<id>` inside the wrapped command then dies with
   `cd: ports/sushigocrafting: No such file or directory` — a scheduling failure, not a code failure.
   Every step here ran with `BUILD_LOCAL=1` and the `cd` inside (the `naturesaura` precedent); a
   *forwarded* SDK step must drop the `cd`.
3. **A mod reaches player verbs through the handle, not the host.** `host.player_give_stack(…)` and
   `host.player_is_secondary_use_active(…)` do not exist on `&mut dyn ServerHost` (those are
   `PlayersBackendMut` methods); the calls are `PlayerRef::give_stack(host, …)` and
   `PlayerRef::is_secondary_use_active(&*host)`. `HostTransaction` (`host.transaction`) and
   `RandomSource` (`next_int_bound`) are *traits to import*, and without them the error is only
   "method not found".
4. **Classify the click through `&dyn Host` first.** `host.tags().key(…)` + `Tags::is_in_tag` are
   `Metadata` access on `&dyn Host`, and the item-id resolution (`ItemKey::id`) too, so `observe()`
   reads the board and the held stack before anything takes `&mut` for the writes. Doing it the other
   way round needs a clone of the stack and a second borrow dance.
5. **A command's level is `invocation.dimension`, not `host.dimension()`.** `execute in` must read and
   write the board in the named dimension, so `command_level` resolves the level from the invocation
   and `write_clicks` takes it — `host.dimension()` is only the server's current level.
6. **Two write shapes, on purpose.** A bare int (`clicks`) goes through
   `BlockEntityRef::merge_data` — it loads, marks the chunk and re-sends the block entity, which is
   what `native_with_mod_block_entity` did — while the item slot goes through `set_item_slot` inside
   `host.transaction`, so the insert and the hand's shrink commit together.
7. **`is_secondary_use_active` is `isShiftKeyDown`, not the crouch pose.** The sneak hand-back must
   read the shift-key flag (`PlayerRef::is_secondary_use_active`), as vanilla's `useWithoutItem`
   path does; the crouching pose (`EntityRef::is_crouching`) is a different question and would miss
   a player who sneaks in the same tick they click.
8. **`component_types` is a *collection* hook that gets the registration context**
   (`fn component_types(&self, cx: &RegistrationContext, event: &mut ComponentTypesEvent<'_>)`), and
   the value still has to be written as a patch: `ItemStack::apply_components(&Compound)` with
   `Tag::Int`, because the SDK has no typed descriptor for a mod's own component. Registering the
   name is still what makes the stack keep it to the client (deviation 3).
9. **The board needs no `BlockEntityPart` at all.** `extradisks` is the precedent: the declared type
   gives the block entity and its empty compound, and the container declaration sizes the storage.
   A Part here would only add a subscription to a tick that does not happen.

### Behaviour that changed with the format (deliberate, and why)

- **The slot limit is the item's own `max_stack_size`**, falling back to 64 when the host cannot say.
  0.1.0 hard-coded 64 (`InventoryComponent`'s default). For every ingredient this slice takes the
  limit is 64, so nothing observable changes; an item with a smaller limit can no longer be
  over-filled into the slot, which upstream's `getMaxStackSize()` slot limit also refused.
- **The give path is `/give`** (`PlayerRef::give_stack`): what does not fit is thrown from the
  player's eyes for them to pick up, where 0.1.0 dropped the overflow at the board. This is closer to
  upstream's `ItemHandlerHelper.giveItemToPlayer`, which drops at the player.
- **The click answers `InteractionResult::Success`** where the retired hook set `*canceled = true`:
  that is the SDK's way of replacing the block's behaviour, and it also keeps the off hand from being
  asked for the same click.
- **No log line** when the cut's amount component cannot be written (0.1.0 had a `log::warn!`): the
  SDK exports no logger and a native mod may not add a crate.
- **`/sushigocrafting` is typed now.** `board <pos:block_pos>` and `clicks <pos:block_pos>
  <clicks:integer>`: the dispatcher refuses a malformed argument before the port sees it, so the
  bespoke `usage:` line only covers a missing/wrong literal. The readout text itself is unchanged,
  byte for byte (`runtime::summary`).

### Gaps after the migration

Everything in the feature gap list above still stands (the other four machines, the crops and the
avocado tree, the fish, the sushi foods and the roller, the mob effects, the container filter, the
guide book, the client renderer). Added by the format:

1. **No in-game run in this environment.** A native `.hmod` is loaded by a game the **launcher** built
   with it (`horizon-launcher native-build ports/sushigocrafting --instance <i> --trust-native-code`),
   and `horizon-launcher` is not installed here (the same wall `imfast`, `enderstorage` and
   `naturesaura` hit). `tools/e2e.sh` is kept and now takes `GAME=` (plus optional `CONTROL_GAME=`)
   with the fail-capable `SELFTEST=1` intact; the 0.1.0 runs in `ports/_shared/evidence/sushigocrafting/`
   (`e2e-default.log`, `e2e-embedded.log`, `e2e-selftest.log`) remain the behaviour's evidence for a
   `runtime.rs` that did not change, and what changed is the *transport* of the same calls.
2. **The content now loads from the package, not from a `--dev-mod` folder.** The 0.1.0 e2e's
   "embedded content, byte-identical" leg has no successor: the file it compared no longer exists.
   Its replacement is structural (`unzip -l` + `check_tags.py` on the delivered `.hmod`) plus the
   launcher's own package load, which is exactly the leg that cannot run here.
3. **The old `--dev-mod content/` route is gone.** A native package cannot be handed to an unmodded
   game, so `SELFTEST`/`CONTROL_GAME` are the only differential levers until a launcher exists.

### Changelog (0.2.0) — supersedes #818

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70)
> retired `.hnmod` and closed **#818** (`sushigocrafting` 0.1.0) with the format purge; this version
> supersedes it. One flat `sushigocrafting.hmod` now carries the code *and* the content (the cutting
> board, its eleven items, the tags including **both** knife spellings, the recipe, the loot table and
> the creative tab): `src/lib.rs` is the one `impl mod_api::Hooks`, `src/board.rs` the glue, and
> `src/runtime.rs` is byte-identical to 0.1.0, so the six recipes, the stroke machine, the amount roll
> and the readout behave exactly as before. Built against SDK `0b14f8a` / mixins `e59da90` for 26.3.
> For `api = "0.2"` the eight old injections became four typed hooks (`component_types`,
> `register_commands`, `run_mod_command` with typed arguments, and `register`), one `Registrar`
> declaration for the block entity type and the one-slot container, and three `BlockPart` methods
> (`state_for_placement`, `use_item_on`, `use_without_item`) where `right_click_block` used to cancel;
> the `.hnmod`, the mixin plan, the per-hook helpers and the embedded-content injection are gone. The
> slot limit is now the item's own `max_stack_size`, the give path is `/give` (overflow thrown at the
> player, as upstream's `giveItemToPlayer` does), and the upstream knife-tag defect stays fixed: the
> cleaver is declared under `c:tools/knife` and `c:tools/knives`, and `tools/check_tags.py` proves it
> in the delivered package. Package: 70 entries, **61,024 B**, sha256
> `44a2a0a9795ed8887acecfe460859342f0ef179bd9e8de95a3e9b8aae188d10e`; `check` and `fits` exit 0.
> **No 0.2.0 in-game run:** the launcher is not installed here (gap 1 above).

### Publication route (campaign ruling 2026-11-18)

The submission form is gone; publishing is now `horizon-rust/horizon-mods#1147` — the artifact is
attached to a release of the author's own repository under the entry's exact `file` name, a PR adds
the `[[versions]]` record and links the file, and a **maintainer** runs `scripts/stage.sh` (a red
`check` before that is the expected `fetch.sh` failure: the reviewed package is not staged yet). The
orchestrator's `python3 ports/tools/publish-ready.py sushigocrafting` does the bookkeeping; this job
owns no submission, and the port stays **local-only** until the maintainer approves publishing
(`ARCHITECTURE.md` §8).

## Review fixes (2026-11-18, PR #1224)

The reviewer found the public cutting-board readout (`/sushigocrafting board <x> <y> <z>`) accepted
arbitrary coordinates and reached `observe`, which read slot zero of any loaded mod block entity,
disclosing that block's item and count without its inventory access policy.

- **Fix.** `readout` requires the block key `sushigocrafting:cutting_board` (`answers_as_a_board`);
  any other block answers `… is not a mod block entity.` before `observe` runs. The
  `/sushigocrafting clicks` harness write is gated the same way.
- **Test.** `the_readout_only_answers_for_the_cutting_board_block` (a furnace, chest, barrel and
  the mod's own unported rice cooker are refused).
- **In-game.** A vanilla chest holding three diamonds is read through the board command and the
  reply is the refusal, with no item id. Evidence:
  `../_shared/evidence/sushigocrafting-e2e/e2e-launcher.log` (`PASS` + `EXIT:0`) and
  `e2e-launcher-control.log` (`EXIT:1`); checker green.
- **Package.** `sushigocrafting.hmod`, **0.2.1**, 61,376 B, sha256
  `c0f681c76551d96e71cb0f1929e72fc7009303e6ba05b9e59ab5d4183ea2c0e6`; two builds byte-identical,
  `check` and `fits --build 0b14f8a` exit 0.


## Slice 2 (0.2.2, 2026-11-21): the fermentation barrel

### The family, and the gap line it comes from

Gap 1 of this file's own list (before this slice):

> **The other four machines**: `rice_cooker` (4 input slots + fuel + an 8,000 mB water tank + a
> 100-tick bar), `fermentation_barrel` (1 in, 1 out, a 2,000 mB tank, 100 ticks), `roller` (the
> weighing/crafting UI) and `cooler_box`.

The **fermentation barrel** is the second of the four and the first machine of this port with a
**fluid** in it. It fits the tier (`content`/`machines`): a block entity, a two-slot container, a
fluid-tank capability, a 100-tick bar, a menu and a screen — no entity, no renderer, no dimension.
It is also the only one of the four whose **own crafting recipe** needs nothing but vanilla items
(`minecraft:furnace`, `#minecraft:planks`, `#minecraft:wooden_pressure_plates`), so the recipe can
be crafted the way it declares in a bare 26.3; the rice cooker's needs `#c:ingots/iron` from
another mod (gap 7, the cleaver's own problem).

Upstream read from the same shallow clone (`Buuz135/SushiGoCrafting`, branch `1.21`, commit
`c3b296bf…`, deleted again after this section): `tile/machinery/FermentationBarrelTile.java`,
`recipe/FermentingBarrelRecipe.java`, `block/machinery/FermentationBarrelBlock.java`,
`SushiContent.Items` (the amount bounds), and the jar's
`data/sushigocrafting/recipe/fermenting_barrel/{cheese,soy}.json`.

### What it does

| Area | Upstream | This port |
|---|---|---|
| the block | `Properties.ofFullCopy(Blocks.OAK_WOOD)` (hardness 2.0, resistance 3.0, wooden), `RotationType.NONE`, `NORMAL` shape (eight boxes) | template `minecraft:barrel` (a container template, so the block opens a menu), the jar's hardness/sound, `occludes: false`, the eight boxes as outline **and** collision |
| the block entity | `ActiveTile`, one entity per block | a declared block entity type; its Part is attached with `block_entity_part`, which also puts the block half on the block |
| the slots | `InventoryComponent("input", 30, 48, 1)` + `("output", 130, 48, 1)`; the input cannot be extracted from, the output cannot be inserted into | a two-slot `ModContainer`, `inputs = [0]`, `outputs = [1]`, every face reaching both |
| the tank | `FluidTankComponent("fluid", 2000, 59, 46)`, `Action.FILL` | `ModFluidTanks` one tank of 2,000 mB, every face reaching it, `inputs = [0]`, `outputs = []`; contents saved under `fluids` |
| the bar | `ProgressBarComponent(93, 48, 100)`, `canIncrease`/`canReset` = `canStart` | `runtime::barrel_step`: count while a batch can start, reset when it cannot, finish at 100 ticks |
| `canStart` | a recipe's input matches, its fluid is in the tank with at least its amount, and the output can take one more (`canStack`) | `runtime::barrel_startable` (`Ingredient.EMPTY` for `"input": []` matches only an **empty** slot) |
| `onFinish` | shrink the input, drain 250 mB, insert/combine the `AmountItem` | `runtime::barrel_finish` + `runtime::combined_amount`, the item through the registered `sushigocrafting:amount` component |
| the recipes | cheese: `"input": []`, milk 250 → `cheese` (50/250/500); soy: `#c:crops/soybean`, water 250 → `soy_sauce` (10/50/100) | `runtime::BARREL_RECIPES`, unit tested against the two JSON files |
| the click | `super.onActivated` (Titanium's fluid-container handling), else `openGui` | a bucket in hand fills the tank (`runtime::bucket_click`); anything else falls through to the block, whose own empty-hand click opens the menu |
| the menu | Titanium's machine panel (2 slots, a tank gauge, the bar) | a menu type `data/sushigocrafting/horizon/menu/fermentation_barrel.json` (2 slots, 2 data slots, slot 1 a result), a `Screen` Part placing them at upstream's 30/130,48 on a 176x166 panel, and the two data slots carrying the bar |
| the tick | `serverTick` (Titanium's bar) | one `block_entity_tick` |

### Two decisions this slice had to make (and why)

1. **Milk is not a fluid Horizon models.** The engine's item-fluid table is explicit about it —
   `horizon-client/src/game/native_item_fluids.rs`: *"The bucket items: (empty, full), by the fluid
   they hold. A bucket is all or nothing, as `BucketItem` is. **Milk, powder snow and fish buckets
   hold no fluid Horizon models.**"* — and `native_fluid_insert` (the pipes' fill) returns the whole
   amount for anything but `minecraft:water`/`minecraft:lava`. NeoForge models `minecraft:milk`, so
   upstream's `FluidUtil.interactWithFluidHandler` fills the tank from a milk bucket; at this pin the
   engine's own helper can only move water and lava. The port therefore moves the vanilla buckets
   itself (`runtime::FULL_BUCKETS`: water, lava, milk — a pure, unit-tested `bucket_click`), writes
   the tank through `merge_data` and swaps one container for the other through
   `ItemPlace::PlayerHand::exchange` in a transaction, as `FluidUtil` does. **What a pipe can do is
   narrower:** the engine's insert moves water and lava only, so milk arrives from a hand (or the
   port's `/sushigocrafting tank` harness command) — a recorded engine gap, not a port choice.
2. **`Action.FILL` is taken at its word.** A container in hand fills the tank; nothing drains it
   into a container (the machine's own `drainForced` takes the batch). This is also what makes the
   click idempotent for the harness: `ctl interact` can be heard twice, and a second click with the
   now-empty bucket is refused instead of emptying what the first one just filled.

### Content carried and written

See [`CONTENT.md`](CONTENT.md) for the file list. In short: the barrel's block model, texture, item
model and 26.3 item definition are carried; its blockstate is **written** (the `minecraft:barrel`
template brings vanilla's `facing`/`open`, so the jar's single `""` variant cannot match — eight
`facing=…,open=…` variants are written instead, as `mekmm`'s barrel-template machines do); its loot
table and crafting recipe are re-spelled for 26.3; `c:crops/soybean` is carried; `soy_bean`, `cheese`
and `soy_sauce` (the last two `AmountItem`s) are declared; the menu type is written.

### Coverage (ruling 2026-11-20) — every family, and what proves it

| Family | Status | Proof |
|---|---|---|
| the barrel block: place, read back, break-drop | **tested** | e2e J: `ctl place` from the item, `ctl block` is `sushigocrafting:fermentation_barrel`; the loot table is carried (one item, `survives_explosion`) |
| its own crafting recipe (7 planks + plate + furnace) | **tested** | e2e I via `tools/craft_barrel.py`: 1 barrel, the grid consumed, and the same ingredients in another pattern → no result |
| the two-slot container and its menu | **tested** | e2e L/M: `ctl interact` opens a 2-slot menu, `ctl click-slot 301` takes the output, `ctl click-slot 300` puts the bean in; e2e J asserts the container reads empty |
| the tank (2,000 mB), a bucket fill, no drain | **tested** | e2e K: milk bucket → `1000/2000`, the bucket swapped; an empty bucket leaves it at 1000; e2e O: the setter clamps 2,500 → 2,000 |
| the 100-tick bar and the batch | **tested** | e2e L: 250 mB + empty slot → one `cheese` with amount 210 (in 50..249), tank 0, bar 0; e2e M: bean + water → `soy_sauce`, bean and 250 mB consumed |
| the cheese recipe (`"input": []`, milk) | **tested** | e2e L (as above); the empty slot is what makes it run |
| the soy recipe (`#c:crops/soybean`, water) | **tested** | e2e M; the tag is carried and the bean is the only member |
| a wrong setup ferments nothing (differential) | **tested** | e2e N: a bean and **no** water, 120 ticks → no output, bar 0 |
| the amount roll of both outputs | **tested** | `runtime::cut_amount` (unit) + the e2e's range assertion, with `SELFTEST=1` inverting it (`FAIL: the cheese amount is 208, expected 250..249`) |
| the menu's tank gauge and progress arrow (drawn) | `not ported` | the screen places the slots and syncs the two data slots; nothing draws the gauge/arrow (no `screen_draw` in this slice — the client half, gap 8 below) |
| fluid in through a **pipe** | `blocked(engine: native_fluid_insert moves water and lava only)` | the tank's `inputs`/`faces` declaration is there for it; a pipe's milk is refused upstream of the port |
| the tank's fluid leaving through the capability | `not ported` (deliberate) | `Action.FILL`: nothing drains a player's tank; the machine's own batch drain is internal |

### The ladder (all inside `build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Evidence |
|---|---|---|---|
| red (tests first, bodies `todo!()`) | `cargo test --offline` | **22 passed, 8 failed** — the eight new barrel tests, `not yet implemented` | `../_shared/evidence/sushigocrafting/sdk-test-red-barrel.log` |
| green | `cargo test --offline` | **30 passed**, 0 failed (19 `runtime::` — 10 board + 9 barrel —, 4 `board::`, 6 `barrel::`, 1 `tests::`) | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 | `sdk-fmt.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **89,472 B**, sha256 `f187e3cf…64bb3` both times, `cmp` identical | `sdk-build.log`, `sdk-build-second.log` |
| registry check | `cargo horizon-mod check sushigocrafting.hmod` | `sushigocrafting 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **89472 bytes**, exit 0 | `sdk-check.log` |
| fits the pin | `cargo horizon-mod fits sushigocrafting.hmod --build 0a875da` | `0a875da  made for` | `sdk-fits.log` |
| fits another build | `… --build 0b14f8a` | `0b14f8a  fits` | `sdk-fits-other.log` |
| gate | `python3 ports/tools/verify-sdk-package.py sushigocrafting` | **1/1** | command output |
| in-game | `launcher-game.sh build sushigocrafting`, then the e2e | `PASS` + `EXIT:0` (every leg A–O) | `../_shared/evidence/sushigocrafting-e2e/e2e-launcher.log` |
| control | the same script on `CONTROL_GAME` (no package) | `FAIL: the board item did not register` + `EXIT:1` | `e2e-launcher-control.log` |
| selftest | `SELFTEST=1` | `FAIL: the cheese amount is 208, expected 250..249` + `EXIT:1` | `e2e-selftest.log` |
| log gate | `check-e2e-logs.py sushigocrafting-e2e --strict` | `logs=3 ok=3 missing-exit=0 bad-exit=0`, exit 0 | command output |

The package's 95 entries are the two manifests, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
`src/{lib,board,barrel,runtime}.rs`, `assets/` (53), `data/` (32) and `uses.json`; **no
`Cargo.lock`**. `uses.json` records **428 surface entries**: 38 hook entries over 14 hooks
(`block_entity`, `block_entity_tick`, `block_entity_ticks`, `blocks`, `command`, `component_types`,
`ingredient`, `items`, `register_commands`, `run_mod_command`, `screen`, `state_for_placement`,
`use_item_on`, `use_without_item`), 6 Part kinds, 44 methods, 99 handles, 27 traits, 211 types, the
two macros and the `registries` capability.

### Traps this slice hit (new to this port)

1. **`ctl screen` / `ctl ui` time out on a `--headless` game.** They need the client's renderer;
   the menu legs therefore run on an `--offscreen` launch (botanypots' e2e is offscreen for the
   same reason). The board's legs were re-run on it unchanged apart from the sneak-click timing
   below.
2. **A sneak click needs the key state to settle.** Under `--offscreen` the first attempt failed
   (`the input after the hand-back is …cucumber`): `ctl key shift down` followed immediately by
   `ctl interact` was heard by the server before the client had the key down. Four ticks either
   side of the click fix it.
3. **A `minecraft:barrel` template inherits `facing` and `open`.** The state reads
   `sushigocrafting:fermentation_barrel[facing=north,open=false]`, so the jar's `""` blockstate
   variant matches nothing; the port writes the eight horizontal variants.
4. **`block_part` + `block_entity_part` is refused.** A Part that implements `BlockEntityPart` must
   be attached with `block_entity_part` only: it also puts the block half on every block of the
   type, which is what makes `use_item_on` reach the barrel. The first version attached it with
   `block_part`, so the bucket click and the tick never ran — the readout was the only thing that
   worked. `the_engine_accepts_every_part_of_this_package` now runs `register` through the SDK's
   own installer, so the engine's verdict is a unit test.
5. **Milk buckets do not stack.** `ctl … item replace … with minecraft:milk_bucket 2` is refused
   (`can only stack up to 1`) and leaves an empty hand, so the click opened the menu instead of
   filling the tank. One bucket per hand.
6. **`<fluid:word>` cannot carry `minecraft:water`.** `word` stops at `:`; the command argument is
   `<fluid:id>`.
7. **A local type named `Item` or `Recipe` makes `fits --build <other>` fail.** The scanner matches
   struct/tuple literals by name against the target build's surface: `Input::Item(…)` and
   `Recipe { … }` were reported as *"the type `Item` can no longer be built with a literal"* on a
   cross-build run even though our types are our own. Renaming them (`Input::Id`,
   `BoardRecipe`) removed the false positive — and made the pure module say what it means.
8. **`ctl interact` must be in reach** (5 blocks) and the player must be on the floor: a helper that
   teleports first and then clicks needs the arena under both.
9. **`ctl container`'s slot numbers are the array index**, not a `slot` field (that reads `null` in
   the reply); `to_entries` gives the index.

### Gaps after slice 2

1. **The other three machines**: `rice_cooker` (4 input slots + fuel + an 8,000 mB water tank + a
   100-tick bar; it produces `cooked_rice`, and its own recipe needs `#c:ingots/iron`, gap 7),
   `roller` (the weighing/crafting UI — the deepest of the four) and `cooler_box` (20 slots, a
   `combine_amount` tick; 20 is not a vanilla menu grid, so it needs a written menu type).
2. **Every crop** (`rice`, `cucumber`, `soy`, `sesame`, `wasabi`) and the **avocado tree**. The
   "content crops do not random-tick" gap was **withdrawn on 2026-11-12** (`ARCHITECTURE.md` §3: a
   content crop on the `wheat` template does grow, `Class::Crop → crop_random_tick`); this port's
   earlier note cited the withdrawn version. A crop slice should re-test it rather than inherit
   the claim. Until then the barrel's `soy_bean` and the board's ingredients come from `ctl give`.
3. **The sources of the ingredients**: `ShrimpEntity`, `TunaEntity`, the `avocado_leaves_ripe`
   drop, the seed loot modifiers (`grass_seeds`, `item_amount`) and the worldgen/biome modifiers.
4. **The container filter for the board.** `container_accepts` is still not a stamped hook in this
   pin's surface (a grep of `surface.json`'s 749 hooks finds none), so a hopper can put any item in
   the board's slot; the barrel has the same hole for its input slot (the machine refuses to run,
   the slot keeps the item).
5. **Steady Hands.** No stamped way to read a player's effects for the amount bonus; the three mob
   effects (`acquired_taste`, `small_bites`, `steady_hands`) are still unregistered.
6. **The sushi foods and the `FoodItem`/`Roller` crafting chain** that consume the AmountItems; the
   `combine_amount` recipe; `nori_sheets`, `tobiko`, `shrimp`, `cooked_rice` and the fish buckets.
   Slice 2 added two of the AmountItems (`cheese`, `soy_sauce`) and the `soy_bean`.
7. **The cleaver-knife and rice-cooker recipes need `#c:ingots/iron`** from another mod: the cleaver
   is carried (recorded, not dropped); the rice cooker will be the same when it is ported.
8. **The client half**: the board's `CuttingBoardRenderer`, and the barrel's tank gauge and progress
   arrow (the screen places the slots and syncs the data slots; nothing draws over them).
9. **The Patchouli guide book** and the 65 `patchouli_books` files.
10. **`combine_amount`, KubeJS scripts, CraftTweaker/JEI/Diet compat.**

### Changelog (0.2.2) — supersedes 0.2.1

> **0.2.2 — the fermentation barrel.** The second of the mod's four machines: a block entity with
> an input and an output slot, a 2,000 mB tank, a 100-tick bar and the GUI Titanium opens, plus the
> jar's two recipes — milk 250 → `cheese` (50..249) and a `#c:crops/soybean` in 250 mB of water →
> `soy_sauce` (10..49). `src/runtime.rs` gained the two recipe rows, the start predicate, the bar,
> the batch, the amount combine and the bucket click, all unit tested; `src/barrel.rs` is the glue
> (declarations, tick, menu + screen, bucket click, readouts) and `src/lib.rs` attaches one Part to
> the block entity type. The content adds the block (on the `minecraft:barrel` template, so it opens
> a menu), its model, texture, loot table and **craftable** recipe (7 planks + a wooden pressure
> plate + a furnace — all vanilla, so it is proven in a real table), the menu type, the
> `c:crops/soybean` tag and three items (`soy_bean`, `cheese`, `soy_sauce`). The tank takes a bucket
> in hand — water, lava **or milk**: `minecraft:milk` is not a fluid Horizon models, so the port
> moves that bucket itself while the engine's `FluidUtil` path handles the rest, and `Action.FILL`
> means a bucket never drains the tank. Built against SDK `0a875da` / mixins `e59da90` for 26.3;
> package **89,472 B**, sha256 `f187e3cf604de5b90586b983cc597728f08b3847cce276d339794d09e8264bb3`,
> 95 entries, `check` and both `fits` runs (the pin and `0b14f8a`) exit 0. In game: `PASS` +
> `EXIT:0` (the barrel is crafted, filled by a milk bucket, one batch of cheese and one of soy sauce
> with their amounts asserted, and a barrel with no water ferments nothing), control `EXIT:1`,
> `SELFTEST=1` `EXIT:1`.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port, rust-review
