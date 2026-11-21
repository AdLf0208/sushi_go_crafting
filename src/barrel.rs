//! The fermentation barrel's glue: the block entity type, the two-slot container with the
//! menu upstream opens, the 2,000 mB tank, the 100-tick bar, the bucket click and the
//! `/sushigocrafting barrel` readout.
//!
//! Every rule lives in [`crate::runtime`] (the two recipes, the start predicate, the bar,
//! the batch and the bucket click); this file reads the game, calls it and writes the
//! result back. Upstream is `FermentationBarrelTile` over Titanium's `ActiveTile`: a fluid
//! tank, an input and an output slot, a progress bar, and a GUI that opens on a click.

use std::array;

use mod_api::{
    self, BlockClick, BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent, BlockPart,
    CommandInvocation, Compound, GameMode, Host, HostTransaction, InteractionResult, ItemPlace,
    ItemResource, ItemStack, LevelRef, ModContainer, ModFluidTanks, RandomSource, Screen,
    ScreenEvent, ServerHost, Tag, UseItemOnEvent,
};

use crate::board::{amount_stack, in_tag};
use crate::runtime::{
    BARREL_TANK_MB, BARREL_TICKS, Bucket, Step, barrel_finish, barrel_startable, barrel_step,
    bucket_click,
};

/// The block (`BlockRegistry.FERMENTATION_BARREL`).
pub(crate) const BARREL: &str = "sushigocrafting:fermentation_barrel";
/// The block entity type's path: `.blocks(&[BARREL_TYPE])` names `<mod id>:fermentation_barrel`.
pub(crate) const BARREL_TYPE: &str = "fermentation_barrel";
/// The menu type `data/sushigocrafting/horizon/menu/fermentation_barrel.json` declares, which
/// the barrel's container opens (`native_block_entity_container`'s `menu`).
pub(crate) const MENU: &str = "sushigocrafting:fermentation_barrel";
/// The one tank's saved entry under `fluids`.
const TANK: usize = 0;
/// `InventoryComponent("input", 30, 48, 1)` and `("output", 130, 48, 1)`.
const INPUT: usize = 0;
const OUTPUT: usize = 1;
const SLOTS: usize = 2;
/// The saved bar (`@Save ProgressBarComponent bar`).
const KEY_PROGRESS: &str = "progress";
/// The empty bucket a filled one becomes (`BucketItem`'s craft remainder).
const BUCKET: &str = "minecraft:bucket";

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// The barrel's block entity type: one entity for the block, starting from an empty compound.
pub(crate) fn declare(registrar: &mut dyn mod_api::Registrar) -> mod_api::BlockEntityTypeKey {
    registrar
        .block_entity_type(BARREL_TYPE)
        .blocks(&[BARREL_TYPE])
        .build()
}

/// The barrel's container: an input and an output slot, every face reaching them, the input
/// the one automation may fill and the output the one it may empty — upstream's two
/// `InventoryComponent`s, whose filters are each other's opposite — and the menu Titanium's
/// `openGui` shows.
pub(crate) fn container() -> ModContainer {
    let all = vec![INPUT, OUTPUT];
    ModContainer {
        slots: SLOTS,
        faces: array::from_fn(|_| all.clone()),
        inputs: vec![INPUT],
        outputs: vec![OUTPUT],
        menu: Some(MENU.to_owned()),
    }
}

/// The barrel's tank: 2,000 mB, every face reaching it, `Action.FILL` — a fill may come from
/// any side, and the machine itself drains it (upstream's `drainForced`).
pub(crate) fn tanks() -> ModFluidTanks {
    let all = vec![TANK];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![i32::try_from(BARREL_TANK_MB).unwrap_or(i32::MAX)];
    tanks.faces = array::from_fn(|_| all.clone());
    tanks.inputs = vec![TANK];
    tanks.outputs = Vec::new();
    tanks
}

/// The menu's layout: upstream's 176 x 166 Titanium panel, with the input at 30,48 and the
/// output at 130,48 as its `InventoryComponent`s place them.
pub(crate) fn screen() -> Screen {
    Screen {
        width: 176.0,
        height: 166.0,
        slots: vec![Some([30.0, 48.0]), Some([130.0, 48.0])],
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
        ..Screen::default()
    }
}

// ---------------------------------------------------------------------------
// The Part
// ---------------------------------------------------------------------------

/// The barrel's Part: the bucket click on the block, and the tick and the screen on the
/// block entity.
pub(crate) struct BarrelPart;

#[mod_api::horizon_part]
impl BlockPart for BarrelPart {
    /// A right click with a bucket in hand: a full one fills the tank (`Action.FILL` — the
    /// barrel never gives a player the fluid back; a batch drains it itself). Anything else
    /// falls through to the block, whose own empty-hand click opens the menu.
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        bucket(host, event.click)
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for BarrelPart {
    /// The barrel ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick of `ActiveTile`'s bar: count it while a batch can start, reset it
    /// when one cannot, and run the batch on the hundredth.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        tick(host, event);
    }

    /// The menu's layout: the two slots where upstream's panel puts them.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        Some(screen())
    }
}

// ---------------------------------------------------------------------------
// The tick
// ---------------------------------------------------------------------------

/// One tick: read the slot, the tank and the bar, ask the pure rules, write back what
/// changed. The menu's two data slots carry the bar to the client (`syncObject(this.bar)`).
fn tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    if event.entity.items.len() < SLOTS {
        event.entity.items.resize(SLOTS, None);
    }
    if event.entity.data.len() < 2 {
        event.entity.data.resize(2, 0);
    }
    let input = item_of(host, &event.entity.items, INPUT);
    let output = item_of(host, &event.entity.items, OUTPUT);
    let tank = read_tank(&event.entity.nbt);
    let progress = read_int(&event.entity.nbt, KEY_PROGRESS);
    let recipe = barrel_startable(
        input.as_ref().map(|(item, _)| item.as_str()),
        tank.as_ref().map(|(fluid, mb)| (fluid.as_str(), *mb)),
        output.as_ref().map(|(item, count)| (item.as_str(), *count)),
        |item, tag| in_tag(host, item, tag),
    );
    let step = barrel_step(progress, recipe.is_some());
    let next = match step {
        Step::Reset => {
            if progress != 0 {
                write_int(&mut event.entity.nbt, KEY_PROGRESS, 0);
            }
            0
        }
        Step::Counting(counted) => {
            write_int(&mut event.entity.nbt, KEY_PROGRESS, counted);
            counted
        }
        Step::Finish => {
            if let Some(recipe) = recipe {
                finish(host, event, recipe, input, output, tank);
            }
            0
        }
    };
    event.entity.data[0] = i32::try_from(next).unwrap_or(i32::MAX);
    event.entity.data[1] = i32::try_from(BARREL_TICKS).unwrap_or(i32::MAX);
}

/// The hundredth tick: drain a batch, shrink the input, and put the rolled output in the
/// output slot, combining into a stack already holding one (`FermentationBarrelTile.onFinish`).
fn finish(
    host: &mut dyn ServerHost,
    event: &mut BlockEntityTickEvent<'_>,
    recipe: &'static crate::runtime::BarrelRecipe,
    input: Option<(String, u32)>,
    output: Option<(String, u32)>,
    tank: Option<(String, u32)>,
) {
    let roll = draw(host, recipe);
    let batch = barrel_finish(
        recipe,
        input.as_ref().map(|(item, count)| (item.as_str(), *count)),
        tank.as_ref().map(|(fluid, mb)| (fluid.as_str(), *mb)),
        output.as_ref().map(|(item, count)| (item.as_str(), *count)),
        roll,
    );
    match batch.input_left {
        Some(left) => {
            if let Some(stack) = event.entity.items.get_mut(INPUT).and_then(Option::as_mut) {
                let _ = stack.set_count(left);
            }
        }
        None => {
            if let Some(slot) = event.entity.items.get_mut(INPUT) {
                *slot = None;
            }
        }
    }
    write_tank(
        &mut event.entity.nbt,
        batch.fluid_left.map(|(fluid, mb)| (fluid.to_owned(), mb)),
    );
    if let Some(stack) = amount_stack(&*host, recipe.output, batch.output_amount)
        && let Some(slot) = event.entity.items.get_mut(OUTPUT)
    {
        *slot = Some(stack);
    }
    write_int(&mut event.entity.nbt, KEY_PROGRESS, 0);
}

/// The amount's roll, from the level's own stream (`world.random.nextInt`), stored back so a
/// mod's draw moves vanilla's later ones as a Java mod's does.
fn draw(host: &mut dyn ServerHost, recipe: &'static crate::runtime::BarrelRecipe) -> u32 {
    let span = recipe.amount.max.saturating_sub(recipe.amount.min).max(1);
    let bound = i32::try_from(span).unwrap_or(i32::MAX);
    let Some(level) = host.level(host.dimension()) else {
        return 0;
    };
    level
        .with_random(host, |random| random.next_int_bound(bound).ok())
        .flatten()
        .map_or(0, |roll| u32::try_from(roll).unwrap_or(0))
}

// ---------------------------------------------------------------------------
// The bucket click
// ---------------------------------------------------------------------------

/// A right click with a bucket: what [`bucket_click`] decides, written to the tank and to
/// the hand in one transaction. A creative player's hand keeps its bucket, as vanilla's
/// `ItemUtils.createFilledResult` leaves it.
fn bucket(host: &mut dyn ServerHost, click: &BlockClick<'_>) -> Option<InteractionResult> {
    let stack = click.stack.filter(|stack| stack.count() > 0)?;
    let level = host.level(click.dimension.key().id())?;
    let pos = click.pos;
    let held = stack.item().id(&*host);
    let entity = host.block_entity(level, pos)?;
    let tank = entity.data(&*host).and_then(|data| read_tank(&data));
    let decision = bucket_click(
        &held,
        tank.as_ref().map(|(fluid, mb)| (fluid.as_str(), *mb)),
    );
    let fluid = match decision {
        Bucket::Fill { fluid } => fluid,
        Bucket::Refused => return Some(InteractionResult::Fail),
        Bucket::NotYours => return None,
    };
    let after = match tank {
        Some((_, mb)) => mb.saturating_add(crate::runtime::BUCKET_MB),
        None => crate::runtime::BUCKET_MB,
    };
    let next_tank = Some((fluid.to_owned(), after));
    let hand = ItemPlace::PlayerHand {
        player: click.player,
        hand: click.hand,
    };
    let creative = host
        .player(click.player)
        .and_then(|player| player.game_mode(&*host))
        == Some(GameMode::Creative);
    let swapped = if creative {
        true
    } else {
        let empty = empty_bucket(&*host)?;
        let mut done = false;
        let _ = host.transaction(|host, tx| {
            done = hand.exchange(host, tx, &ItemResource::of(&empty), 1) == 1;
            Ok::<(), ()>(())
        });
        done
    };
    if !swapped {
        return Some(InteractionResult::Fail);
    }
    write_tank_into(host, level, pos, next_tank);
    Some(InteractionResult::Success)
}

/// An empty bucket, count one.
fn empty_bucket(host: &dyn Host) -> Option<ItemStack> {
    let id = mod_api::Identifier::parse(BUCKET).ok()?;
    ItemStack::new(host.item_key(&id)?, 1)
}

/// Write the tank through `merge_data`, which marks the chunk, so a client's copy agrees.
fn write_tank_into(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    tank: Option<(String, u32)>,
) {
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    let mut patch = Compound::new();
    write_tank(&mut patch, tank);
    let _ = entity.merge_data(host, patch);
}

// ---------------------------------------------------------------------------
// Saved state
// ---------------------------------------------------------------------------

fn read_int(nbt: &Compound, key: &str) -> u32 {
    match nbt.get(&key.into()) {
        Some(Tag::Int(value)) => u32::try_from(*value).unwrap_or(0),
        _ => 0,
    }
}

fn write_int(nbt: &mut Compound, key: &str, value: u32) {
    nbt.insert(
        key.into(),
        Tag::Int(i32::try_from(value).unwrap_or(i32::MAX)),
    );
}

/// The tank's contents from the saved compound: the declaration's shape is one entry per
/// tank under `fluids`, each `{Fluid: "<id>", Amount: <mB>}`.
pub(crate) fn read_tank(nbt: &Compound) -> Option<(String, u32)> {
    let Some(Tag::List(entries)) = nbt.get(&"fluids".into()) else {
        return None;
    };
    let Some(Tag::Compound(entry)) = entries.get(TANK) else {
        return None;
    };
    let fluid = match entry.get(&"Fluid".into()) {
        Some(Tag::String(fluid)) if fluid.to_string_lossy() != "minecraft:empty" => {
            fluid.to_string_lossy()
        }
        _ => return None,
    };
    let amount = match entry.get(&"Amount".into()) {
        Some(Tag::Int(amount)) => u32::try_from((*amount).max(0)).unwrap_or(0),
        _ => 0,
    };
    (amount > 0).then_some((fluid, amount))
}

/// Write the tank back in the same shape, clamped to the declaration's capacity.
pub(crate) fn write_tank(nbt: &mut Compound, tank: Option<(String, u32)>) {
    let (fluid, amount) = match tank {
        Some((fluid, mb)) => (fluid, mb.min(BARREL_TANK_MB)),
        None => ("minecraft:empty".to_owned(), 0),
    };
    let mut entry = Compound::new();
    entry.insert("Fluid".into(), Tag::String(fluid.as_str().into()));
    entry.insert(
        "Amount".into(),
        Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
    );
    nbt.insert("fluids".into(), Tag::List(vec![Tag::Compound(entry)]));
}

/// Slot `index` of a block entity's items, with its count, as an owned pair.
fn item_of(host: &dyn Host, items: &[Option<ItemStack>], index: usize) -> Option<(String, u32)> {
    let stack = items.get(index)?.as_ref()?;
    (stack.count() > 0).then(|| (stack.item().id(host), stack.count()))
}

// ---------------------------------------------------------------------------
// The `/sushigocrafting barrel` and `tank` commands
// ---------------------------------------------------------------------------

/// `/sushigocrafting barrel <pos>`: the readout line, `runtime::barrel_summary`.
pub(crate) fn readout(
    host: &mut dyn ServerHost,
    invocation: &CommandInvocation,
    pos: [i32; 3],
) -> String {
    let Some(level) = crate::board::command_level(&*host, invocation) else {
        return format!("{pos:?} is not a mod block entity.");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(String::new, |key| key.id(&*host));
    // The readout reads slot zero and the saved compound of whatever stands here: require this
    // mod's own barrel first, so another block's inventory is never reported (the review fix on
    // the board's command).
    if block != BARREL {
        return format!("{pos:?} is not a mod block entity.");
    }
    let Some(entity) = host.block_entity(level, pos) else {
        return format!(
            "{block} at {} {} {}: no block entity",
            pos[0], pos[1], pos[2]
        );
    };
    let slot = |index: usize| {
        entity
            .item_slot(&*host, index as u32)
            .ok()
            .flatten()
            .filter(|stack| stack.count() > 0)
            .map(|stack| (stack.item().id(&*host), stack.count()))
    };
    let input = slot(INPUT);
    let data = entity.data(&*host).unwrap_or_default();
    let tank = read_tank(&data);
    let progress = read_int(&data, KEY_PROGRESS);
    let next = crate::runtime::barrel_recipe(
        input.as_ref().map(|(item, _)| item.as_str()),
        tank.as_ref().map(|(fluid, mb)| (fluid.as_str(), *mb)),
        |item, tag| in_tag(&*host, item, tag),
    );
    crate::runtime::barrel_summary(
        pos,
        input.as_ref().map(|(item, count)| (item.as_str(), *count)),
        tank.as_ref().map(|(fluid, mb)| (fluid.as_str(), *mb)),
        progress,
        next,
    )
}

/// `/sushigocrafting tank <pos> <fluid> <mb>`: set the tank, clamped to the declaration's
/// 2,000 mB. A harness control like `/setblock`, so a leg can put milk in a tank the engine's
/// item-fluid model cannot carry (see `notes.md`).
pub(crate) fn set_tank(
    host: &mut dyn ServerHost,
    invocation: &CommandInvocation,
    pos: [i32; 3],
    fluid: &str,
    mb: i32,
) {
    let Some(level) = crate::board::command_level(&*host, invocation) else {
        return;
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(String::new, |key| key.id(&*host));
    if block != BARREL {
        return;
    }
    let mb = u32::try_from(mb.max(0))
        .unwrap_or(u32::MAX)
        .min(BARREL_TANK_MB);
    write_tank_into(host, level, pos, (mb > 0).then(|| (fluid.to_owned(), mb)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_is_the_two_slot_menu_upstream_opens() {
        let container = container();
        assert_eq!(container.slots, SLOTS);
        for face in &container.faces {
            assert_eq!(*face, vec![INPUT, OUTPUT], "every face reaches both");
        }
        assert_eq!(container.inputs, vec![INPUT], "automation fills the input");
        assert_eq!(
            container.outputs,
            vec![OUTPUT],
            "automation empties the output"
        );
        assert_eq!(container.menu.as_deref(), Some(MENU), "Titanium's openGui");
    }

    #[test]
    fn the_tank_is_two_thousand_fillable_millibuckets() {
        let tanks = tanks();
        assert_eq!(tanks.tanks, vec![2_000]);
        assert_eq!(tanks.inputs, vec![TANK], "a fill may come from any side");
        assert!(
            tanks.outputs.is_empty(),
            "upstream's Action.FILL drains nothing through the capability"
        );
        for face in &tanks.faces {
            assert_eq!(*face, vec![TANK]);
        }
    }

    #[test]
    fn the_screen_puts_the_slots_where_upstreams_panel_does() {
        let screen = screen();
        assert_eq!((screen.width, screen.height), (176.0, 166.0));
        assert_eq!(screen.slots, vec![Some([30.0, 48.0]), Some([130.0, 48.0])]);
        assert_eq!(screen.slots.len(), SLOTS);
    }

    #[test]
    fn the_tank_round_trips_through_the_saved_compound() {
        let mut nbt = Compound::new();
        assert_eq!(read_tank(&nbt), None, "a fresh barrel holds nothing");
        write_tank(&mut nbt, Some(("minecraft:milk".to_owned(), 1_250)));
        assert_eq!(read_tank(&nbt), Some(("minecraft:milk".to_owned(), 1_250)));
        write_tank(&mut nbt, None);
        assert_eq!(read_tank(&nbt), None);
        // The declaration clamps: a saved 2,500 reads back as the tank's 2,000.
        write_tank(&mut nbt, Some(("minecraft:water".to_owned(), 9_999)));
        assert_eq!(read_tank(&nbt), Some(("minecraft:water".to_owned(), 2_000)));
    }

    #[test]
    fn a_tank_holding_the_empty_marker_reads_empty() {
        let mut entry = Compound::new();
        entry.insert("Fluid".into(), Tag::String("minecraft:empty".into()));
        entry.insert("Amount".into(), Tag::Int(500));
        let mut nbt = Compound::new();
        nbt.insert("fluids".into(), Tag::List(vec![Tag::Compound(entry)]));
        assert_eq!(read_tank(&nbt), None);
    }

    #[test]
    fn the_barrel_and_its_menu_are_named_after_the_mod() {
        assert_eq!(BARREL, "sushigocrafting:fermentation_barrel");
        assert_eq!(BARREL_TYPE, "fermentation_barrel");
        assert_eq!(MENU, "sushigocrafting:fermentation_barrel");
    }
}
