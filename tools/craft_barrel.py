#!/usr/bin/env python3
"""Craft the fermentation barrel's own recipe in a real table.

Called by `tools/e2e.sh` with the open game's control socket. The recipe is the
jar's `data/sushigocrafting/recipe/fermentation_barrel.json`
(`IGI`/`IFI`/`III` with `I` = `#minecraft:planks`, `G` =
`#minecraft:wooden_pressure_plates`, `F` = `minecraft:furnace`), re-spelled for
26.3 by `tools/assemble_content.py`. Uses the harness's crafting grid
(`ctl screen`: slots 100-108 row-major, result 200), gives the ingredients and
asserts, in this order:

1. seven oak planks, one oak pressure plate and one furnace in the declared
   pattern craft exactly one `sushigocrafting:fermentation_barrel`; the result
   is shift-clicked into the inventory and the counts are read back, so the
   assertion covers the output id, its count and that the grid was consumed;
2. the same ten ingredients in a pattern the recipe does not declare (the
   furnace beside the pressure plate instead of under it) craft nothing — the
   control, so an empty result is the recipe's answer and not a dead grid.

The table is replaced before each case, so an untaken result cannot leak into
the next grid. Exits 1 with the observed result on the first mismatch.
"""
import json
import subprocess
import sys

# `IGI` / `IFI` / `III` row-major over slots 100..108.
PLANK_SLOTS = ["100", "102", "103", "105", "106", "107", "108"]
PLATE_SLOT = ["101"]
FURNACE_SLOT = ["104"]
# The control: the furnace where the pattern's left plank is, so no row matches.
WRONG_FURNACE_SLOT = ["100"]
WRONG_PLANK_SLOTS = ["102", "103", "104", "105", "106", "107", "108"]


def main():
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args):
        # A bounded reply: a `ctl` call with no timeout blocks forever, and the
        # kill then looks like a pass (ports/_shared/launcher-e2e.md, trap 8).
        out = subprocess.run(
            [ctl, "--socket", sock, "--timeout", "30", *args],
            text=True,
            capture_output=True,
        )
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def slots():
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def find(item):
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} in the inventory")

    def result():
        """The result slot's stack, or None: an empty slot has `item: null`."""
        stack = [s for s in slots() if s["slot"] == 200]
        if not stack or not stack[0].get("item"):
            return None
        return stack[0]

    def place(item, targets):
        slot = find(item)
        run("click-slot", str(slot))
        wait(1)
        for target in targets:
            run("click-slot", str(target), "right")
            wait(1)
        run("click-slot", str(slot))
        wait(2)

    def setup(items):
        try:
            run("close")
        except SystemExit:
            pass
        wait(5)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, count in items:
            run("give", item, str(count))
        wait(3)
        run("interact", x, y, z)
        wait(5)
        for _ in range(10):
            screen = run("screen")
            if "slots" in screen:
                return
            wait(2)
        raise SystemExit(f"the crafting table did not open a menu: {screen}")

    def counts():
        return run("inventory").get("counts", {})

    # `IGI`/`IFI`/`III` is seven planks, one plate and one furnace: nine cells.
    ingredients = [
        ("minecraft:oak_planks", 7),
        ("minecraft:oak_pressure_plate", 1),
        ("minecraft:furnace", 1),
    ]
    failures = []

    def expect(label, placements, want_item, want_count):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"{label} ->", json.dumps(got))
        if not got or got.get("item") != want_item or got.get("count") != want_count:
            failures.append(f"{label}: expected {want_count} {want_item}, got {got}")
            return
        run("click-slot", "200", "shift")
        wait(3)
        run("close")
        wait(3)
        got_counts = counts()
        print("  inventory after taking it ->", json.dumps(got_counts))
        if got_counts.get(want_item, 0) != want_count:
            failures.append(
                f"{label}: the inventory holds {got_counts.get(want_item, 0)} {want_item}, "
                f"expected {want_count}"
            )
        for item, _ in ingredients:
            if got_counts.get(item, 0) != 0:
                failures.append(
                    f"{label}: {got_counts.get(item, 0)} {item} left, so the grid was not consumed"
                )

    def expect_empty(label, placements):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"{label} ->", json.dumps(got))
        if got is not None:
            failures.append(f"{label}: control crafted {got.get('count')} {got.get('item')}")

    # The control first: every case starts with `clear @s`, and the crafted
    # barrel must be left in the inventory for the caller's own legs.
    expect_empty(
        "the same ingredients with the furnace where the left plank goes",
        [
            ("minecraft:oak_planks", WRONG_PLANK_SLOTS),
            ("minecraft:oak_pressure_plate", PLATE_SLOT),
            ("minecraft:furnace", WRONG_FURNACE_SLOT),
        ],
    )
    expect(
        "7 planks, 1 pressure plate, 1 furnace in the declared pattern",
        [
            ("minecraft:oak_planks", PLANK_SLOTS),
            ("minecraft:oak_pressure_plate", PLATE_SLOT),
            ("minecraft:furnace", FURNACE_SLOT),
        ],
        "sushigocrafting:fermentation_barrel",
        1,
    )

    run("close")
    wait(3)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        raise SystemExit(1)
    print(
        "recipe PASS: the carried barrel recipe crafts exactly one "
        "sushigocrafting:fermentation_barrel from 7 planks, a wooden pressure plate "
        "and a furnace, and consumes them; the same ingredients in another pattern "
        "craft nothing"
    )


if __name__ == "__main__":
    main()
