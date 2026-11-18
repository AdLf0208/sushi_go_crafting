#!/usr/bin/env python3
"""Prove both knife tags ship with the cleaver, in the delivered `.hmod`.

Upstream's `CuttingBoardTile` tests `c:tools/knife` (singular) while its own data
ships only `data/c/tags/item/tools/knives.json` (plural), so the mod's own
cleaver cannot cut on its own board unless another mod supplies the singular tag
(`notes.md`, "Upstream defects fixed"). The port declares the cleaver under both
names; this asserts the delivered bytes do too, so a package that drops one file
fails here instead of silently reproducing the defect.

    python3 tools/check_tags.py             # the delivered sushigocrafting.hmod
    python3 tools/check_tags.py --selftest  # prove the check can fail

Exit 0 when both tags hold the cleaver in the package, 1 otherwise.
"""
import argparse
import io
import json
from pathlib import Path
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
# The one flat SDK package: code and content in the same file, built by
# `cargo horizon-mod build` and copied to the port root (the deliverable).
PACKAGE = ROOT / "sushigocrafting.hmod"

# Both spellings upstream's ecosystem uses: the one its code tests and the one
# its data declares.
TAGS = (
    "data/c/tags/item/tools/knife.json",
    "data/c/tags/item/tools/knives.json",
)
KNIFE = "sushigocrafting:cleaver_knife"


def tag_values(archive: zipfile.ZipFile, name: str) -> list[str] | None:
    """The tag's values, or None when the archive does not carry the file."""
    try:
        raw = archive.read(name)
    except KeyError:
        return None
    parsed = json.loads(raw)
    values = parsed.get("values", [])
    return [value for value in values if isinstance(value, str)]


def check(archive: zipfile.ZipFile, label: str, echo: bool) -> list[str]:
    """Every tag in `archive` lists the cleaver; return the problems found."""
    problems = []
    for name in TAGS:
        values = tag_values(archive, name)
        if values is None:
            problems.append(f"{label}: missing {name}")
            continue
        if KNIFE not in values:
            problems.append(f"{label}: {name} does not list {KNIFE} (values {values})")
        elif echo:
            print(f"  {label}: {name} = {values}")
    return problems


def check_bytes(data: bytes, label: str, echo: bool) -> list[str]:
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        return check(archive, label, echo)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--selftest", action="store_true",
                        help="run the check on copies with the defect put back; it must fail")
    args = parser.parse_args()

    if not PACKAGE.is_file():
        print(f"FAIL: missing {PACKAGE} (build it: rm -f Cargo.lock && "
              f"cargo horizon-mod build, then copy it here)", file=sys.stderr)
        return 1

    package = PACKAGE.read_bytes()
    problems = []
    print(f"== knife tags in {PACKAGE.relative_to(ROOT)} ==")
    problems += check_bytes(package, PACKAGE.name, echo=True)

    if args.selftest:
        # Put the upstream defect back in memory and make sure the same check
        # rejects it: without this, a passing run could be a check that never
        # looks at anything.
        print("== selftest: the defect must be caught ==")
        with zipfile.ZipFile(io.BytesIO(package)) as archive:
            without = {name: archive.read(name) for name in archive.namelist()}
        singular = TAGS[0]
        if singular not in without:
            print(f"  (no {singular} to remove: the real check already failed, as it must)")
        else:
            stripped = {name: data for name, data in without.items() if name != singular}
            caught = check_bytes(_zip(stripped), "without the singular tag", echo=False)
            if not caught:
                print("FAIL: the check accepted a package with no singular tag", file=sys.stderr)
                return 1
            print(f"  caught: {caught[0]}")
            emptied = dict(without)
            emptied[singular] = json.dumps({"values": []}).encode()
            caught = check_bytes(_zip(emptied), "without the cleaver", echo=False)
            if not caught:
                print("FAIL: the check accepted a singular tag without the cleaver", file=sys.stderr)
                return 1
            print(f"  caught: {caught[0]}")
            print("  selftest: both defect shapes are rejected")

    if problems:
        for problem in problems:
            print(f"FAIL: {problem}", file=sys.stderr)
        return 1
    print(f"PASS: both {TAGS[0]} and {TAGS[1]} ship with {KNIFE} in {PACKAGE.name}")
    return 0


def _zip(files: dict[str, bytes]) -> bytes:
    """A zip of `files`, in memory, for the selftest's defective copies."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            archive.writestr(name, data)
    return buffer.getvalue()


if __name__ == "__main__":
    sys.exit(main())
