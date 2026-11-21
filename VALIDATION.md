# Validation — Sushi Go Crafting slice 1 (the cutting board)

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6). Every step ran inside
the campaign's build-slot registry (`ports/tools/build-slot.sh`), never a bare
`cargo`, never a `--target-dir`, `CARGO_INCREMENTAL=0`.

Evidence lives in `ports/_shared/evidence/sushigocrafting/`.

## 1. The pure runtime (red → green)

```
$ cargo test --locked --offline
```
- **red** (tests written first, bodies `todo!()`): `1 passed; 8 failed` —
  `test-red.log`.
- **green** (implemented): `10 passed; 0 failed` — `test-green.log`, and the
  final run in `ladder.log`:
  ```
  running 10 tests
  test tests::a_cut_gives_the_jar_amount_for_the_roll ... ok
  test tests::five_strokes_count_and_the_sixth_cuts ... ok
  test tests::a_single_stack_stops_at_max_combine ... ok
  test tests::the_jars_cleaver_is_a_knife_and_a_stick_is_not ... ok
  test tests::a_slot_takes_only_its_own_item_and_only_what_fits ... ok
  test tests::the_board_turns_toward_its_placer ... ok
  test tests::a_non_ingredient_is_not_a_recipe_input ... ok
  test tests::an_item_and_a_tag_both_find_their_recipe ... ok
  test tests::the_six_recipes_are_the_jars ... ok
  test tests::the_readout_names_the_input_the_strokes_and_the_next_output ... ok
  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```

## 2. Style and lints (`ladder.log`)

- `cargo fmt --check` — clean.
- `rustfmt --check --edition 2024 mixins/sushigocrafting/*.rs` — clean (the glue
  files are injected modules, not crate sources, so they are checked directly).
- `cargo clippy --all-targets --locked --offline -- -D warnings` —
  `Checking horizon-sushigocrafting v0.1.0` → finished, **0 warnings**.

## 3. The staged ladder

| Step | Command | Result | Log |
|---|---|---|---|
| stage | `python3 tools/horizon.py stage` | plan accepted, `stage_changed: true` | `ladder-stage-check-test.log` |
| check | `python3 tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** | `ladder-stage-check-test.log` |
| test | `python3 tools/horizon.py test` | **10 passed** as `game::sushigocrafting::tests` (`1240 filtered out`), `cargo_exit: 0` | `embedded-tests.log` (copy of `target/horizon-test.log`) |
| build | `python3 tools/horizon.py build` | `cargo_exit: 0`, fast profile (rebuilt in every e2e lease) | e2e logs |

## 4. Packages (`package-run1.log`, `package-run2.log`)

```
content/sushigocrafting.hmod size=36600 sha256=417fe7b210a178693cfed5d259c501401561beb4a47ca7fe10240c186d9a134b entries=64
sushigocrafting.hnmod        size=42467 sha256=71de4a4201c06976ce6bd51718991191051d0d7ab2c256205bfedc9e3e592b88 entries=15
```

- `content/sushigocrafting.hmod` — the content mod, built by the pin's own
  `cargo horizon-mod build`: *“no code: content only”*, 64 files (62 content
  files + the `LICENSE` and `LICENSE-NOTICE.md` that `tools/notice.py` puts
  inside the archive, the campaign-wide notice pattern).
- `sushigocrafting.hnmod` — the one deliverable, in the port root, with the built
  `.hmod` embedded at `mixins/sushigocrafting/data/sushigocrafting-content.hmod`
  and the licence notices beside its sources.
- `cargo horizon-mod check sushigocrafting.hnmod --kind native-mod`:
  `native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3`, unsigned, and the
  registry check reports it as unsandboxed native source (it is not executed).
- **Both knife tags ship** in the `.hmod` and in the bundle, each listing the
  cleaver (`tools/check_tags.py`, run by `tools/package.sh` and by every e2e leg).
- **Reproducible:** two consecutive `tools/package.sh` runs produced the same
  two SHA-256 values (`package-run1.log`, `package-run2.log`).

## 5. In-game (fail-capable e2e)

All three legs launched the fast build in a scratch home
(`mktemp -d /tmp/opencode/sushigocrafting-home.XXXXXX`), The End arena at
x=200, seed 20261105, with the mod's own commands read through `ctl chat`.

| Leg | Command | Result | Log |
|---|---|---|---|
| dev content (`--dev-mod content/`) | `build-slot.sh e2e … -- tools/e2e.sh` | `PASS` + `EXIT:0` | `e2e-default.log` |
| embedded (no `--dev-mod`), with `CONTROL=1` | `… -- env EMBEDDED=1 tools/e2e.sh` | control: no board item, block stays `minecraft:air`; embedded file 36,600 B byte-identical; `PASS` + `EXIT:0` | `e2e-embedded.log` |
| selftest | `… -- env SELFTEST=1 tools/e2e.sh` | `FAIL: the slice amount is 192, expected 200..199` + `EXIT:1` | `e2e-selftest.log` |

Every leg begins with the **structural** check (`tools/check_tags.py`), before a
world is opened: both `data/c/tags/item/tools/knife.json` and
`data/c/tags/item/tools/knives.json` are present in `content/sushigocrafting.hmod`
and in the `.hmod` embedded in `sushigocrafting.hnmod`, and each lists
`sushigocrafting:cleaver_knife`. A package that reproduced upstream's defect
(missing singular tag, or a tag without the cleaver) fails here — the pre-fix
package is `tags-check-red.log`, and `tags-check-selftest.log` proves both defect
shapes are rejected.

The scenario (every line in the logs):

- **A** the board registers and places from its item; the placed block is
  `sushigocrafting:cutting_board[subfacing=west]` (the placer stands west of it);
  the readout of an empty board is `input=none clicks=0/6 next=none`.
- **B** a cucumber (2) goes from the hand into the slot: `input=…:cucumber x2`,
  `next=…:cucumber_slices 50..199`, and the hand no longer holds it.
- **C control** seven clicks with a **stick** leave `clicks=0`, the input at 2
  and no slices — a non-knife strokes nothing.
- **D** one knife click counts 1–2 strokes (the harness's press/release can be
  heard twice) and cuts nothing; the stroke count is then set to exactly 5 and
  the readout shows `clicks=5/6` with the input intact.
- **E** the sixth stroke cuts exactly one: input 2→1, `clicks` back to 0 (or 1,
  from the same command's second use), one `sushigocrafting:cucumber_slices`
  whose components read `{"sushigocrafting:amount":149}` — inside 50..199.
- **F** a sneak with an empty hand returns the remaining cucumber to the player
  and empties the slot.
- **G control** dirt is refused from the hand; inserted through the container it
  sits in the slot with `next=none` and six knife strokes give nothing.

`python3 ports/tools/check-e2e-logs.py sushigocrafting --strict`:

```
ok       sushigocrafting        e2e-default.log            PASS + EXIT:0
ok       sushigocrafting        e2e-embedded.log           PASS + EXIT:0
ok       sushigocrafting        e2e-selftest.log           FAIL + EXIT:1

logs=3  ok=3  missing-exit=0  bad-exit=0
```

## 6. Gates

- `python3 ports/tools/check-e2e-logs.py sushigocrafting --strict` → exit 0
  (3/3 logs, `0` missing exits).
- `python3 ports/tools/check-skills.py --notes ports/sushigocrafting` →
  `SKILLS GATE: PASS`.
- `python3 ports/tools/upstream_ready.py sushigocrafting` → `READY` (MIT), exit 0.
- `python3 ports/tools/upstream_publish.py sushigocrafting` (dry run) → the
  publish tree is sanitised and `ready`, its `registry.toml` sha equals the
  delivered `sushigocrafting.hnmod` (`71de4a42…`) and the live port's.
- `PUBLISH_SPACING=30 ports/tools/publish_ready.sh sushigocrafting` → the gate
  passes, then **`GITHUB_TOKEN is not set`**: the token is passed per command by
  the orchestrator and stored nowhere, so the PR #677 push is the orchestrator's
  step (`publish-dryrun.log`, `publish-attempt.log`).

## 7. Upstream analysis hygiene

The analysis clone `ports/_shared/upstream/sushigocrafting` (branch `1.21`,
commit `c3b296b`) was deleted after `FINDINGS.md` was written; the jar stays in
`ports/_shared/licence-cache/` as the licence evidence. No Java was copied and no
class file was decompiled.

## 8. Migration to the SDK format (0.2.0, 2026-11-18)

**Sections 1–7 above describe the retired `.hnmod` format.** Horizon's mod-API merge (#16941,
launcher ≥ 0.2.70) retired the container and closed the port's former submission **#818**; the
0.1.0 artifacts (`content/sushigocrafting.hmod` 36,600 B, `sushigocrafting.hnmod` 42,467 B, the
staged `tools/horizon.py` ladder, the embedded-content leg) are historical evidence of the
behaviour, now in `ports/_shared/evidence/sushigocrafting/retired-hnmod-0.1.0/`.

The 0.2.0 package is one flat SDK `.hmod` (api 0.2), pinned to nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4` / mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Every step below ran in a `build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`;
`logs` are under `ports/_shared/evidence/sushigocrafting/sdk-*`.

| Step | Command | Result | Evidence |
|---|---|---|---|
| runtime tests before any glue | `cargo test --offline` | **10 passed**, 0 failed | `sdk-test-runtime-before-glue.log` |
| tests | `cargo test --offline` | **13 passed** (10 `runtime::`, 3 `board::`), 0 failed | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0, clean | `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/sushigocrafting.hmod` | `sdk-build.log` |
| registry check | `cargo horizon-mod check sushigocrafting.hmod` | `sushigocrafting 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, **61024 bytes**, sha256 `44a2a0a9…8d10e`, **exit 0** | `sdk-check.log` |
| fits the pinned build | `cargo horizon-mod fits sushigocrafting.hmod --build nightly` | `Sushi Go Crafting:` / `0b14f8a  made for`, **exit 0** | `sdk-fits.log` |
| reproducible | `cargo horizon-mod build` again | 61,024 B, `44a2a0a9…8d10e` both times; the delivered file matches a rebuild from the current sources | `sdk-reproducible.txt`, `sdk-build-second.log` |
| `runtime.rs` untouched | `unzip -p … src/runtime.rs \| sha256sum` | `ef62dee6…a4197` = `src/runtime.rs` = the retired file | `sdk-runtime-sha256.txt` |
| entries | `unzip -l sushigocrafting.hmod` | 70 entries: the two manifests, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `src/` (3), `assets/` (37), `data/` (24), `uses.json`; **no `Cargo.lock`** | `sdk-entries.log` |
| what it uses | `unzip -p … uses.json` | **335 surface entries**: 25 hook entries over 10 hooks, 35 methods, 5 Part kinds, 74 handles, 26 traits, 167 types, 2 macros, the `registries` capability | `sdk-uses-from-package.json`, `sdk-uses-summary.txt` |
| content carry | `diff -r` against the retired `content/{assets,data}`, then a `assemble_content.py` re-run | no differences; the 61 files reproduce byte for byte | `sdk-content-carry.log` |
| knife tags (the upstream defect stays fixed) | `python3 tools/check_tags.py`, `--selftest` | `PASS` — both tags in the delivered package; the selftest rejects both defect shapes | `sdk-tags-check.log` |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/sushigocrafting --strict` | `SKILLS GATE: PASS` | command output |

### Not run: the in-game e2e (and why)

A native `.hmod` is loaded by a game the **launcher** built with it
(`horizon-launcher native-build ports/sushigocrafting --instance <i> --trust-native-code`), and
`horizon-launcher` is not installed in this environment. `tools/e2e.sh` is kept, now takes
`GAME=` (a launcher-built game) and an optional `CONTROL_GAME=` (an unmodded game where both ids
must be unknown), and keeps the fail-capable `SELFTEST=1`. **No 0.2.0 in-game leg is claimed.** The
0.1.0 runs (`e2e-default.log`, `e2e-embedded.log`, `e2e-selftest.log`) stand as the behaviour's
evidence for a `runtime.rs` that did not change; what changed is the transport of the same calls.

## 9. Slice 2 — the fermentation barrel (0.2.2), pin `0a875da` (2026-11-21)

Everything below ran in a `build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`
(the runner forwards `check`/`build` to the remote box otherwise, whose own `cd` then eats the
wrapped `cd ports/sushigocrafting`).

| Step | Command | Result | Evidence |
|---|---|---|---|
| red (tests first, bodies `todo!()`) | `cargo test --offline` | **22 passed, 8 failed** — the eight barrel tests, `not yet implemented` | `../_shared/evidence/sushigocrafting/sdk-test-red-barrel.log` |
| tests | `cargo test --offline` | **30 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 | `sdk-fmt.log` |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | **89,472 B**, sha256 `f187e3cf604de5b90586b983cc597728f08b3847cce276d339794d09e8264bb3`, `cmp` identical, 95 entries, no `Cargo.lock` | `sdk-build.log`, `sdk-build-second.log` |
| registry check | `cargo horizon-mod check sushigocrafting.hmod` | `sushigocrafting 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **89472 bytes**, exit 0 | `sdk-check.log` |
| fits the pin | `cargo horizon-mod fits sushigocrafting.hmod --build 0a875da` | `0a875da  made for` | `sdk-fits.log` |
| fits another build | `… --build 0b14f8a` | `0b14f8a  fits` | `sdk-fits-other.log` |
| package gate | `python3 ports/tools/verify-sdk-package.py sushigocrafting` | **1/1** | command output |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/sushigocrafting --strict` | `SKILLS GATE: PASS` | command output |

### The in-game leg (offscreen, launcher-built game)

The package is a native `.hmod`, so the game is the launcher's build of it
(`ports/tools/launcher-game.sh build sushigocrafting` → `GAME`/`CTL`/`CONTROL_GAME`/`RES`
exports). The leg is **`--offscreen`**, not `--headless`: `ctl screen`/`ctl ui` need the client's
renderer and time out without one, and the menu legs open the barrel's GUI.

| Leg | What it asserts | Log |
|---|---|---|
| main | `PASS` + `EXIT:0` — A–H the board (unchanged), I the barrel's recipe crafted in a real table (1 barrel, grid consumed, another pattern → nothing), J place + empty readout, K a milk bucket fills to 1000/2000 and an empty bucket gives none back, L one batch of cheese (amount 210, inside 50..249), tank 250→0, bar back to 0, taken out through the menu's 2 slots, M a soy bean through the menu's input + 250 mB of water → `soy_sauce`, bean and water consumed, N a bean with no water and 120 ticks → nothing, O the tank setter clamps 2,500 → 2,000 | `../_shared/evidence/sushigocrafting-e2e/e2e-launcher.log` |
| control | the same script on `CONTROL_GAME` (the Nightly without the package): `FAIL: the board item did not register` + `EXIT:1` | `…/e2e-launcher-control.log` |
| selftest | `SELFTEST=1` (the cheese amount range inverted to 250..249): `FAIL: the cheese amount is 208, expected 250..249` + `EXIT:1` | `…/e2e-selftest.log` |

`python3 ports/tools/check-e2e-logs.py sushigocrafting-e2e --strict`:

```
ok       sushigocrafting-e2e    e2e-launcher-control.log   FAIL + EXIT:1
ok       sushigocrafting-e2e    e2e-launcher.log           PASS + EXIT:0
ok       sushigocrafting-e2e    e2e-selftest.log           FAIL + EXIT:1

logs=3  ok=3  missing-exit=0  bad-exit=0
```

`SELFTEST=board` inverts the board's slice amount instead (kept from slice 1); the recorded
selftest run is this slice's family.
