# Licence notice — Sushi Go Crafting for Horizon

## The original mod

- **Sushi Go Crafting** `1.21-0.6.7` by Buuz135 —
  <https://github.com/Buuz135/SushiGoCrafting>, branch `1.21`, commit
  `c3b296bf81b4f2a712ea1a3c58cbef0f86746f55`.
- **Licence: MIT**, read from the jar's `META-INF/neoforge.mods.toml`:
  `ports/_shared/licence-cache/sushigocrafting-1.21-0.6.7.jar`
  (SHA-256 recorded by `ports/tools/audit_licences.py`; the jar is kept as the
  licence evidence).
- Copyright (c) Buuz135 and the Sushi Go Crafting contributors, under the MIT
  terms reproduced in [`LICENSE`](LICENSE).

## What this repository carries from it

The **assets and data** of the cutting board and the eleven items its loop needs
— blockstate, block model, textures, item models, the lang keys that name them,
the six item tags, the crafting recipe and the loot table. Which files, and every
re-spelling for 26.3, is listed in [`CONTENT.md`](CONTENT.md). The content
carry is the original project's MIT-licensed material, redistributed under the
same licence with this notice.

## What is a rewrite

`mixins/sushigocrafting/runtime.rs` and `support.rs` are a **clean-room Rust
rewrite** of the machine's behaviour, written from the upstream repository read
as documentation (the files are named in [`FINDINGS.md`](FINDINGS.md)) and from
the jar's data. No Java source was copied or translated line by line, and no
class file was decompiled.

## Publishing

The maintainer's approval covers publishing permissive and copyleft ports
(`ARCHITECTURE.md` §8). This port is **MIT (rank 0)**, so it may be published
with this notice and the original copyright.

## Upstream repository clone

The analysis clone lived at `ports/_shared/upstream/sushigocrafting` (gitignored,
never committed or shipped) and was deleted once `FINDINGS.md` was written.
