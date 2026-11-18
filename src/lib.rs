//! Sushi Go Crafting for Horizon — slice 1: the cutting board, one flat SDK package.
//!
//! Ported from Sushi Go Crafting 1.21-0.6.7 (`Buuz135/SushiGoCrafting`, MIT); see
//! [`FINDINGS.md`](https://github.com/Buuz135/SushiGoCrafting) for the behaviour read
//! from the jar and the upstream repository, and `notes.md` for the hook map, the gaps
//! and the migration to the SDK format.
//!
//! [`runtime`] is the pure truth — the jar's six recipes, the knife-stroke state machine,
//! the amount roll, the insertion rule, the placement yaw mapping and the readout line —
//! and is byte-identical to the retired `.hnmod` port's `runtime.rs`. [`board`] is the
//! glue: a block Part (the click and the placement state), the block entity type and the
//! one-slot container declarations, the saved stroke count, and the `/sushigocrafting`
//! commands. `assets/` and `data/` are the carried content; the whole mod is this one
//! `.hmod`.
//!
//! # Where the retired container's hooks went
//!
//! The 2026-11-17 mod-API merge (#16941, launcher >= 0.2.70) retired `.hnmod`: the plan,
//! the injection points and the per-hook helper files are gone, and a hook is a typed
//! method or, for a moment that belongs to one object, a Part. The mapping is:
//!
//! | `.hnmod` hook (API) | here |
//! |---|---|
//! | `component_types` (65) | [`Hooks::component_types`] |
//! | `block_entity` (15) | the block entity **type** declared in [`Hooks::register`] |
//! | `block_entity_container` (16) | [`Registrar::block_container`] in [`Hooks::register`] |
//! | `state_for_placement` (76) | [`board::BoardPart::state_for_placement`] (a `BlockPart`) |
//! | `right_click_block` (21) | [`board::BoardPart::use_item_on`] / `use_without_item` |
//! | `register_commands` (8) | [`Hooks::register_commands`] |
//! | `perform_command` (8) | [`Hooks::run_mod_command`], on typed arguments |
//! | the `content_root` injection (AE2 pattern) | gone: one flat `.hmod` carries `assets/` and `data/` itself |

pub mod board;
pub mod runtime;

use mod_api::{
    self, ComponentTypesEvent, Flow, Hooks, RegisterCommandsEvent, Registrar, RegistrationContext,
    RunModCommandEvent, ServerHost,
};

/// The mod has no per-session state: a board's whole state is its block entity's slot and
/// saved stroke count, and the recipe table is a constant.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the board: its block entity type (one block entity for the block, starting
    /// from an empty compound) and its one-slot container, then attach the Part that
    /// answers the placement state and the click.
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        let block = registrar.block_key(board::BOARD);
        registrar.block_container(&block, board::container());
        // A refusal is the game's to report at load: without the type the board keeps no
        // storage, and the click and the readout then find nothing.
        let _ = board::declare(registrar);
        registrar.block_part(block, board::BoardPart);
    }

    /// Before content loads: register `sushigocrafting:amount`, the `AmountItem`
    /// component, so a cut's stack carries it through saves and the network and the game
    /// knows its type (an unregistered component is kept as an unknown entry and does not
    /// survive to the client — measured in game, `notes.md` deviation 3).
    fn component_types(&self, _cx: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(board::AMOUNT.to_owned());
    }

    /// `/sushigocrafting board|clicks <x> <y> <z>`: the readout and the harness's stroke
    /// setter. Upstream registers no commands of its own.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        board::declare_commands(event);
    }

    /// The command bodies. The dispatcher has already parsed and resolved every argument,
    /// so nothing here splits a line; the readout text is the retired port's, byte for
    /// byte, because the existing in-game assertions match on it.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        if event.command.literals.first().map(String::as_str) != Some(board::COMMAND) {
            return Flow::Continue;
        }
        let line = board::run_command(host, event.command);
        board::feedback(host, &line);
        Flow::Cancel
    }
}
