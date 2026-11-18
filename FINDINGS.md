# Sushi Go Crafting — findings (slice 1: the cutting board)

Ported from **Sushi Go Crafting 1.21-0.6.7** (`Buuz135/SushiGoCrafting`, **MIT**),
Minecraft Java 1.21.1 / NeoForge. Behaviour read from the upstream repository at
branch `1.21`, commit **`c3b296bf81b4f2a712ea1a3c58cbef0f86746f55`** (2026-10-27,
"Increased version" — the jar's build day), cloned shallow into
`ports/_shared/upstream/sushigocrafting` and deleted after this file was written.
Assets and data are read from the exact ATM10 jar
`ports/_shared/licence-cache/sushigocrafting-1.21-0.6.7.jar` (MIT read from
`META-INF/neoforge.mods.toml`; see `../_shared/licence-audit.csv`).

No decompilation: the jar was opened as an archive (`unzip -l`, `unzip -p`) for
its assets/data and licence; the classes were not read.

## The jar, measured (`unzip -l`: 640 entries)

| Family | What is in the jar |
|---|---|
| blocks (14) | `cutting_board`, `rice_cooker`, `fermentation_barrel`, `roller`, `cooler_box`; crops `rice_crop`, `cucumber_crop`, `soy_crop`, `sesame_crop`, `wasabi_crop`; `avocado_log`, `avocado_leaves`, `avocado_leaves_logged`, `avocado_sapling` |
| items (74 model files) | seeds (`*_seeds`), plant produce (`rice`, `cucumber`, `soy_bean`, `wasabi_root`, `avocado`, `sesame_seed`), AmountItems (`cucumber_slices`, `avocado_slices`, `imitation_crab`, `salmon_fillet`, `tuna_fillet`, `wasabi_paste`, `cooked_rice`, `cheese`, `soy_sauce`, `tobiko`, `shrimp`, `sesame_seed`), sushi foods (`*_maki`, `*_nigiri`, `*_temaki`, `*_california`, `*_gunkan`, `onigiri`), `cleaver_knife`, `nori_sheets`, `seaweed_on_a_stick`, `guide_book`, mob buckets |
| recipes (24) | 6 `cutting_board/`, 2 `fermenting_barrel/`, plus the crafting recipes of the blocks/items and `combine_amount` |
| loot tables (19) | one per block, plus `entities/shrimp`, `entities/tuna` |
| tags (49) | `c:` crops/fruits/seeds/raw_fishes/tools/knives, `minecraft:` crops/leaves/logs/saplings/bee_growables/mineable, `diet:`, `sushigocrafting:pressing_base` |
| worldgen (5) | the avocado tree (configured/placed feature) and two biome modifiers (avocado, shrimp/tuna spawns) |
| assets | 15 blockstates, 38 block models, 74 item models, 135 textures, 4 lang files, a Patchouli guide book (65 files) |
| other | 2 entities (`ShrimpEntity`, `TunaEntity`), 3 mob effects (`acquired_taste`, `small_bites`, `steady_hands`), 2 global loot modifiers (`grass_seeds`, `item_amount`), KubeJS examples, CraftTweaker/JEI/Diet compat |

## What the cutting board does (upstream)

`tile/machinery/CuttingBoardTile.java` (whole class read) —
`block/machinery/CuttingBoardBlock.java`, `recipe/CuttingBoardRecipe.java`,
`datagen/SushiRecipeProvider.java`, `proxy/SushiContent.java`,
`api/impl/FoodIngredient.java`, `item/AmountItem.java`.

- **The block** is `Properties.ofFullCopy(Blocks.OAK_TRAPDOOR)` (hardness 3.0,
  blast resistance 3.0, wooden sounds, no occlusion), a Titanium
  `RotatableBlock` with `RotationType.FOUR_WAY`: the state property is
  **`subfacing`** (north/south/east/west), and the shape is
  `Block.box(1, 0, 3, 15, 1, 13)` on north/south and
  `Block.box(3, 0, 1, 13, 1, 15)` on east/west. The jar's own blockstate selects
  on `subfacing` with `y` rotations 0/90/180/270.
- **One item slot** (`InventoryComponent("input", 0, 0, 1)`), an input filter
  that accepts an item only when **some cutting-board recipe's input matches it**
  (`accepts`), no output slot and **no menu** (there is no `openGui` call and no
  screen class; the GUI texture in the jar is the roller's).
- **The interaction** (`onActivated`, right click):
  1. holding an item, board non-empty, the held item is in
     `c:tools/knife` → `++click`; when `click > 5` **one cut** happens, `click`
     resets to 0, and the click returns `SUCCESS`;
  2. holding an item, board empty, the item is a recipe input → the **whole held
     stack** is inserted (`stack.setCount(0)`), `SUCCESS`;
  3. empty hand and **sneaking** → the board's stack goes back to the player,
     `SUCCESS`;
  4. nothing else → `FAIL` (the click falls through).
- **One cut** (`click > 5`): for every recipe whose input matches the board's
  stack, give the player the recipe's ingredient item and `shrink(1)` the board's
  stack. The item is
  `FoodAPI.get().getIngredientFromName(recipe.getIngredient()).getItem()`; for an
  `AmountItem` it is `AmountItem.random(player, level)` — a stack of one with the
  `sushigocrafting:amount` component set to
  `min(getMaxCombineAmount(), extra + rand.nextInt(max - min) + min)`, where
  `extra` is `(steady_hands amplifier + 1) * min` when the player has Steady
  Hands and 0 otherwise.
- **No tick work**: the block entity has a server tick but does nothing in it
  (all the state is the slot and the click count, both `@Save`d).

### The jar's cutting-board recipes (all six, `data/sushigocrafting/recipe/cutting_board/*.json`)

| Recipe file | input | ingredient | output item | amount (min/max/combine) |
|---|---|---|---|---|
| `avocado.json` | `#c:fruits/avocado` | `avocado` | `sushigocrafting:avocado_slices` | 100 / 500 / 1000 |
| `crab.json` | `minecraft:cod` | `crab` | `sushigocrafting:imitation_crab` | 50 / 200 / 400 |
| `cucumber.json` | `#c:crops/cucumber` | `cucumber` | `sushigocrafting:cucumber_slices` | 50 / 200 / 400 |
| `salmon.json` | `#c:raw_fishes/salmon` | `salmon` | `sushigocrafting:salmon_fillet` | 500 / 2000 / 4000 |
| `tuna.json` | `#c:raw_fishes/tuna` | `tuna` | `sushigocrafting:tuna_fillet` | 1000 / 3000 / 6000 |
| `wasabi.json` | `#c:crops/wasabi_root` | `wasabi` | `sushigocrafting:wasabi_paste` | 10 / 50 / 100 |

The recipe→item mapping is `FoodIngredient` (name → item) and the amounts are the
`amountItem(id, category, min, max, combine, hurts)` calls in
`SushiContent.Items`. All six are AmountItems: **stack size 1**, food nutrition
**2**, saturation modifier **0.3**, poison (100 ticks, amplifier 0) at
probability **0.01**, or **0.6** for the `hurts` ones (`wasabi_paste`,
`soy_sauce`). `cleaver_knife` and the plant produce are plain items (stacks of
64).

The tag contents in the jar are one line each and name the mod's own ids
(`c:crops/cucumber` → `sushigocrafting:cucumber`, `c:fruits/avocado` →
`sushigocrafting:avocado`, `c:crops/wasabi_root` → `sushigocrafting:wasabi_root`,
`c:raw_fishes/tuna` → `sushigocrafting:raw_tuna`), except `c:raw_fishes/salmon`
which names vanilla's `minecraft:salmon`.

### An upstream inconsistency the port has to decide (documented, not silent)

`CuttingBoardTile.KNIFE` is `c:tools/knife` (**singular**), but the jar's data
ships `data/c/tags/item/tools/knives.json` (**plural**), holding
`sushigocrafting:cleaver_knife`. In a pack where nothing else defines
`c:tools/knife` (Farmer's Delight does, in ATM10), the mod's **own knife cannot
cut on its own board**. The port checks the **union** of `c:tools/knife` and
`c:tools/knives`: it keeps upstream's code tag (other mods' knives still work)
and makes the jar's own knife work, which is what the acronym-less data tag was
for. No upstream-valid case is lost; the e2e proves the jar's knife cuts and a
non-knife does not.

## The reference ports read for the shape

- `ports/industrialforegoing` — the closest precedent: a Titanium machine at this
  pin, `subfacing` from `state_for_placement`, a block entity, harness commands,
  content embedded in the `.hnmod` (the AE2 pattern), and its e2e style.
- `ports/botanypots` — a block entity with a container slot, a menu, and the
  `native_with_mod_block_entity` write guard; its notes carry the measured
  **content-crop growth gap** this port cites instead of re-deriving
  (`ARCHITECTURE.md` §3: a content crop on the `wheat` template does not
  random-tick, and a `random_tick` injection never fires for a content block).
- `ports/charginggadgets`, `ports/laserio` — the `content_root` embedding helper.

## What this slice does not touch (recorded in `notes.md` as gaps)

The rice cooker, fermentation barrel, roller and cooler box; all five crops and
the avocado tree; the two fish entities; the sushi foods and the roller's
weighing/crafting UI; the three mob effects; the seed loot modifiers; the
worldgen and biome modifiers; the Patchouli book; the crafting board's client
renderer (the item on the board and the knife animation) and its `subfacing`
orientation, which a headless run cannot check.
