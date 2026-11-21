#!/usr/bin/env python3
"""Build the package's `assets/` + `data/` for the cutting board and the barrel from the ATM10 jar.

Carries the board's and the barrel's blockstate, block model and texture, the
items' models and textures, and the lang strings that name them; re-spells the
loot tables and the crafting recipes in 26.3's shape; and writes the
declarations this campaign owns (the blocks, the items, the menu, the creative
tab, the tags).

Every decision is recorded in `CONTENT.md`. Rerun it to rebuild: it deletes the
`assets/sushigocrafting/` and `data/` trees it owns first, so the output is
reproducible.

    python3 tools/assemble_content.py [--jar PATH]
"""
import argparse
import json
import shutil
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared/licence-cache/sushigocrafting-1.21-0.6.7.jar"
NS = "sushigocrafting"

# The two machines and the items their loops need: the board's six recipe inputs
# and six outputs (`FINDINGS.md`), and the barrel's one tag input and two
# outputs (`recipe/fermenting_barrel/*.json`).
BLOCK = "cutting_board"
BARREL = "fermentation_barrel"
PLAIN_ITEMS = ["cucumber", "avocado", "wasabi_root", "raw_tuna", "cleaver_knife", "soy_bean"]
# (name, min, max, combine, poison probability) from `SushiContent.Items`
# `amountItem(...)` calls and `AmountItem`'s food builder.
AMOUNT_ITEMS = [
    ("avocado_slices", 100, 500, 1000, 0.01),
    ("cucumber_slices", 50, 200, 400, 0.01),
    ("imitation_crab", 50, 200, 400, 0.01),
    ("salmon_fillet", 500, 2000, 4000, 0.01),
    ("tuna_fillet", 1000, 3000, 6000, 0.01),
    ("wasabi_paste", 10, 50, 100, 0.6),
    ("cheese", 50, 250, 500, 0.01),
    ("soy_sauce", 10, 50, 100, 0.6),
]
ITEMS = PLAIN_ITEMS + [entry[0] for entry in AMOUNT_ITEMS]

# The board's shape, `Block.box(1, 0, 3, 15, 1, 13)` for north/south and
# `Block.box(3, 0, 1, 13, 1, 15)` for east/west (`CuttingBoardBlock`).
SHAPE_NS = [[0.0625, 0.0, 0.1875, 0.9375, 0.0625, 0.8125]]
SHAPE_EW = [[0.1875, 0.0, 0.0625, 0.8125, 0.0625, 0.9375]]

# The barrel's shape: the eight boxes `FermentationBarrelBlock.NORMAL` unions (the staves
# and the base, the hoop, the two uprights and the two posts), in block units over 16.
BARREL_SHAPE = [
    [2 / 16, 1 / 16, 1 / 16, 14 / 16, 15 / 16, 2 / 16],
    [2 / 16, 1 / 16, 14 / 16, 14 / 16, 15 / 16, 15 / 16],
    [2 / 16, 0.0, 2 / 16, 14 / 16, 1 / 16, 14 / 16],
    [2 / 16, 14.5 / 16, 2 / 16, 14 / 16, 15.5 / 16, 14 / 16],
    [1 / 16, 1 / 16, 2 / 16, 2 / 16, 15 / 16, 14 / 16],
    [14 / 16, 1 / 16, 2 / 16, 15 / 16, 15 / 16, 14 / 16],
    [4 / 16, 15 / 16, 6 / 16, 5 / 16, 1.0, 10 / 16],
    [11 / 16, 15 / 16, 6 / 16, 12 / 16, 1.0, 10 / 16],
]

# The item tags for these items (`data/c/tags/item/**`). Upstream's code tests
# `c:tools/knife` (singular) while its own data ships only `c:tools/knives`
# (plural) holding the cleaver, so its cleaver cannot cut on its own board
# without another mod supplying the singular tag. The port declares the cleaver
# under BOTH names (the fix, `notes.md` "Upstream defects fixed") and the runtime
# accepts either (`runtime::KNIFE_TAGS`).
TAGS = {
    "c/tags/item/crops/cucumber": ["sushigocrafting:cucumber"],
    "c/tags/item/fruits/avocado": ["sushigocrafting:avocado"],
    "c/tags/item/crops/wasabi_root": ["sushigocrafting:wasabi_root"],
    "c/tags/item/raw_fishes/tuna": ["sushigocrafting:raw_tuna"],
    "c/tags/item/raw_fishes/salmon": ["minecraft:salmon"],
    "c/tags/item/tools/knives": ["sushigocrafting:cleaver_knife"],
    "c/tags/item/tools/knife": ["sushigocrafting:cleaver_knife"],
    "c/tags/item/crops/soybean": ["sushigocrafting:soy_bean"],
}


def read(jar, path):
    with zipfile.ZipFile(jar) as archive:
        return archive.read(path)


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n")


def carry(jar, source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(read(jar, source))


def block_declaration():
    return {
        "template": "minecraft:oak_planks",
        "properties": {
            "hardness": 3.0,
            "explosion_resistance": 3.0,
            "requires_tool": False,
            "sounds": "minecraft:oak_planks",
            "occludes": False,
        },
        "state_properties": {
            "subfacing": {"type": "enum", "values": ["north", "south", "east", "west"]}
        },
        "default_state": {"subfacing": "north"},
        "shapes": {
            "property": "subfacing",
            "values": {
                "north": SHAPE_NS,
                "south": SHAPE_NS,
                "east": SHAPE_EW,
                "west": SHAPE_EW,
            },
        },
        "collision_shapes": {
            "property": "subfacing",
            "values": {
                "north": SHAPE_NS,
                "south": SHAPE_NS,
                "east": SHAPE_EW,
                "west": SHAPE_EW,
            },
        },
    }


def barrel_declaration():
    # `FermentationBarrelBlock`: `Properties.ofFullCopy(Blocks.OAK_WOOD)` (hardness 2.0,
    # blast resistance 3.0, wooden sounds), `RotationType.NONE` (no state property), and
    # `NORMAL` for both shapes. The template is vanilla's barrel so the block opens a menu
    # as a container does; `item: true` gives it the item that places it.
    return {
        "template": "minecraft:barrel",
        "properties": {
            "hardness": 2.0,
            "explosion_resistance": 3.0,
            "requires_tool": False,
            "sounds": "minecraft:oak_wood",
            "occludes": False,
        },
        "item": True,
        "shapes": {"default": BARREL_SHAPE},
        "collision_shapes": {"default": BARREL_SHAPE},
    }


def barrel_blockstate():
    # The jar's own state is `{"variants": {"": …}}` because upstream's block has no
    # properties (`RotationType.NONE`). The `minecraft:barrel` template it is declared on
    # carries vanilla's `facing` and `open`, so the state must be spelled out for them:
    # the model is symmetric about its axis, so a `y` rotation changes nothing visible.
    y = {"north": None, "east": 90, "south": 180, "west": 270}
    variants = {}
    for facing, rotation in y.items():
        for open_ in ("false", "true"):
            variant = {"model": f"{NS}:block/{BARREL}"}
            if rotation is not None:
                variant["y"] = rotation
            variants[f"facing={facing},open={open_}"] = variant
    return {"variants": variants}


def barrel_menu():
    # The two slots and the two progress data slots (`native_block_entity_container`'s menu);
    # the output is a result slot, which players take from and automation reaches as `outputs`.
    return {"slots": 2, "data_slots": 2, "results": [1]}


def plain_item_declaration():
    return {"template": "minecraft:paper"}


def amount_item_declaration(poison):
    # `AmountItem`: stacksTo(1), nutrition 2, saturationModifier 0.3 (26.3's
    # absolute saturation is nutrition * modifier * 2 = 1.2), poison 100 ticks
    # at amplifier 0 with the item's own probability.
    return {
        "template": "minecraft:apple",
        "max_stack_size": 1,
        "components": {
            "minecraft:food": {"nutrition": 2, "saturation": 1.2},
            "minecraft:consumable": {
                "on_consume_effects": [
                    {
                        "type": "minecraft:apply_effects",
                        "effects": [
                            {
                                "id": "minecraft:poison",
                                "duration": 100,
                                "amplifier": 0,
                                "show_icon": True,
                            }
                        ],
                        "probability": poison,
                    }
                ]
            },
        },
    }


def loot_table(block=BLOCK):
    # 26.3: one typed condition on the pool (the jar's 1.21.1 `conditions` list).
    return {
        "type": "minecraft:block",
        "pools": [
            {
                "condition": {"type": "minecraft:survives_explosion"},
                "entries": [{"type": "minecraft:item", "name": f"{NS}:{block}"}],
                "rolls": 1,
            }
        ],
        "random_sequence": f"{NS}:blocks/{block}",
    }


def board_recipe():
    # The jar's `recipe/cutting_board.json`, re-spelled: 26.3 ingredients are a
    # bare item id or `#tag`, and the `neoforge:item_exists` gate is dropped
    # (the item exists whenever this mod does).
    return {
        "type": "minecraft:crafting_shaped",
        "category": "misc",
        "key": {"B": "#minecraft:logs", "S": "#minecraft:slabs"},
        "pattern": ["   ", "SSS", "BBB"],
        "result": {"count": 1, "id": f"{NS}:{BLOCK}"},
    }


def barrel_recipe():
    # The jar's `recipe/fermentation_barrel.json`, re-spelled: `#tag` for a tag, a bare item
    # id otherwise, and the `neoforge:item_exists` gate dropped. Its ingredients are all
    # vanilla's (`minecraft:furnace`, planks, wooden pressure plates), so it is craftable in
    # a bare 26.3 — unlike the cleaver's `#c:ingots/iron`.
    return {
        "type": "minecraft:crafting_shaped",
        "category": "misc",
        "key": {
            "F": "minecraft:furnace",
            "G": "#minecraft:wooden_pressure_plates",
            "I": "#minecraft:planks",
        },
        "pattern": ["IGI", "IFI", "III"],
        "result": {"count": 1, "id": f"{NS}:{BARREL}"},
    }


def knife_recipe():
    return {
        "type": "minecraft:crafting_shaped",
        "category": "misc",
        "key": {"I": "#c:ingots/iron", "S": "minecraft:stick"},
        "pattern": [" II", "II ", "S  "],
        "result": {"count": 1, "id": f"{NS}:cleaver_knife"},
    }


def lang(jar):
    carried = json.loads(read(jar, f"assets/{NS}/lang/en_us.json"))
    keys = [
        f"block.{NS}.{BLOCK}",
        f"block.{NS}.{BARREL}",
        "itemGroup.sushigocrafting",
    ]
    keys += [f"item.{NS}.{name}" for name in ITEMS]
    return {key: carried[key] for key in keys}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    args = parser.parse_args()
    jar = args.jar
    # The package root is the content root now (the SDK layout): `assets/` and
    # `data/` sit beside `src/` and the one manifest, and there is no separate
    # `content/` folder with its own manifest to write.
    content = ROOT
    for stale in (content / "assets" / NS, content / "data"):
        shutil.rmtree(stale, ignore_errors=True)

    # Carried byte-for-byte from the jar.
    carry(jar, f"assets/{NS}/blockstates/{BLOCK}.json",
          content / "assets" / NS / "blockstates" / f"{BLOCK}.json")
    for block in (BLOCK, BARREL):
        carry(jar, f"assets/{NS}/models/block/{block}.json",
              content / "assets" / NS / "models/block" / f"{block}.json")
        carry(jar, f"assets/{NS}/textures/block/{block}.png",
              content / "assets" / NS / "textures/block" / f"{block}.png")
        write_json(
            content / "assets" / NS / "models/item" / f"{block}.json",
            json.loads(read(jar, f"assets/{NS}/models/item/{block}.json")),
        )
        write_json(
            content / "assets" / NS / "items" / f"{block}.json",
            {"model": {"type": "minecraft:model", "model": f"{NS}:item/{block}"}},
        )
    for name in ITEMS:
        model = json.loads(read(jar, f"assets/{NS}/models/item/{name}.json"))
        write_json(content / "assets" / NS / "models/item" / f"{name}.json", model)
        for texture in model.get("textures", {}).values():
            _, path = texture.split(":", 1)
            carry(jar, f"assets/{NS}/textures/{path}.png",
                  content / "assets" / NS / "textures" / f"{path}.png")
        # 26.3 item model definition: `assets/<ns>/items/<name>.json`.
        write_json(
            content / "assets" / NS / "items" / f"{name}.json",
            {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}},
        )

    write_json(content / "assets" / NS / "blockstates" / f"{BARREL}.json", barrel_blockstate())
    write_json(content / "assets" / NS / "lang/en_us.json", lang(jar))

    # Written here: the declarations, the 26.3 spellings and the slice's tags.
    write_json(content / "data" / NS / "horizon/block" / f"{BLOCK}.json", block_declaration())
    write_json(content / "data" / NS / "horizon/block" / f"{BARREL}.json", barrel_declaration())
    write_json(content / "data" / NS / "horizon/menu" / f"{BARREL}.json", barrel_menu())
    for name in PLAIN_ITEMS:
        write_json(content / "data" / NS / "horizon/item" / f"{name}.json",
                   plain_item_declaration())
    for name, _min, _max, _combine, poison in AMOUNT_ITEMS:
        write_json(content / "data" / NS / "horizon/item" / f"{name}.json",
                   amount_item_declaration(poison))
    write_json(
        content / "data" / NS / "horizon/creative_tab" / "sushigocrafting.json",
        {
            "icon": f"{NS}:{BLOCK}",
            "items": [f"{NS}:{BLOCK}", f"{NS}:{BARREL}"]
            + [f"{NS}:{name}" for name in ITEMS],
            "title": "itemGroup.sushigocrafting",
        },
    )
    write_json(content / "data" / NS / "loot_table/blocks" / f"{BLOCK}.json", loot_table())
    write_json(content / "data" / NS / "loot_table/blocks" / f"{BARREL}.json",
               loot_table(BARREL))
    write_json(content / "data" / NS / "recipe" / f"{BLOCK}.json", board_recipe())
    write_json(content / "data" / NS / "recipe" / f"{BARREL}.json", barrel_recipe())
    write_json(content / "data" / NS / "recipe/cleaver_knife.json", knife_recipe())
    for path, values in TAGS.items():
        write_json(content / "data" / Path(path).with_suffix(".json"), {"values": values})
    write_json(
        content / "data/minecraft/tags/block/mineable/axe.json",
        {"values": [f"{NS}:{BLOCK}", f"{NS}:{BARREL}"]},
    )

    files = sorted(
        path
        for tree in (content / "assets", content / "data")
        for path in tree.rglob("*")
        if path.is_file()
    )
    print(f"content: {len(files)} files under {content}/assets and {content}/data")


if __name__ == "__main__":
    main()
