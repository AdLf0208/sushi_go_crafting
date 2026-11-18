//! The cutting board's glue: the block Part (the click and the placement state), the
//! declarations, the saved state and the `/sushigocrafting` command bodies.
//!
//! Every rule lives in [`crate::runtime`] (the retired `.hnmod`'s `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result back.
//! Its tests pin what it *declares* (one slot every face reaches, no menu) and how the
//! saved stroke count reads back.

use mod_api::{
    self, BlockClick, BlockPart, BlockState, Command, CommandArgumentValue, CommandInvocation,
    Compound, Hand, Host, HostTransaction, Identifier, InteractionResult, ItemKey, ItemPlace,
    ItemResource, ItemStack, LevelHost, LevelRef, ModContainer, PlayerRef, RandomSource,
    RegisterCommandsEvent, Registrar, ServerHost, StateForPlacementEvent, Tag, Text,
    UseItemOnEvent, UseWithoutItemEvent,
};

use crate::runtime::{
    Recipe, Stroke, accepts, cut_amount, facing_for_yaw, fits, is_knife, recipe_for, stroke,
    summary,
};

/// The board (`BlockRegistry.CUTTING_BOARD`).
pub(crate) const BOARD: &str = "sushigocrafting:cutting_board";
/// The block entity type's path, in this mod's namespace: `.blocks(&[BOARD_TYPE])` names
/// `<mod id>:cutting_board`, the same block `BOARD` names.
pub(crate) const BOARD_TYPE: &str = "cutting_board";
/// The state property Titanium's `RotationType.FOUR_WAY` sets, which the jar's own
/// blockstate selects on.
const SUBFACING: &str = "subfacing";
/// The board's saved stroke count (`CuttingBoardTile.click`, `@Save int click`).
const KEY_CLICKS: &str = "clicks";
/// The one container slot (`InventoryComponent("input", 0, 0, 1)`).
const INPUT_SLOT: usize = 0;
const SLOT_COUNT: usize = 1;
/// The slot limit when the item's own maximum cannot be read: upstream's component
/// default is the item's `getMaxStackSize`, 64 for every ingredient this slice takes.
const MAX_STACK: u32 = 64;
/// The component an `AmountItem` carries (`SushiDataComponent.AMOUNT`).
pub(crate) const AMOUNT: &str = "sushigocrafting:amount";
/// The mod command's root word.
pub(crate) const COMMAND: &str = "sushigocrafting";

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// The block entity type the board uses: what the retired `block_entity` hook returned —
/// one block entity for the block, starting from an empty compound (so a fresh board has
/// no `clicks` key and the slot is empty).
pub(crate) fn declare(registrar: &mut dyn Registrar) -> mod_api::BlockEntityTypeKey {
    registrar
        .block_entity_type(BOARD_TYPE)
        .blocks(&[BOARD_TYPE])
        .build()
}

/// The board's container: one slot every face reaches, and no menu — upstream has an
/// `InventoryComponent` and never calls `openGui`.
pub(crate) fn container() -> ModContainer {
    ModContainer {
        slots: SLOT_COUNT,
        faces: std::array::from_fn(|_| vec![INPUT_SLOT]),
        inputs: vec![INPUT_SLOT],
        outputs: vec![INPUT_SLOT],
        menu: None,
    }
}

// ---------------------------------------------------------------------------
// The block Part: the placement state and the click
// ---------------------------------------------------------------------------

/// The board's Part, attached to the block: a placed board faces its placer, and a right
/// click is the machine's one interaction.
pub(crate) struct BoardPart;

#[mod_api::horizon_part]
impl BlockPart for BoardPart {
    /// A placed board faces its placer (`Titanium`'s `FOUR_WAY`), as the retired
    /// `state_for_placement` injection set it.
    fn state_for_placement(
        &self,
        level: &dyn LevelHost,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        let faced =
            level.state_with_property(event.state, SUBFACING, facing_for_yaw(event.context.yaw))?;
        Some(Some(faced))
    }

    /// A right click with something in hand: insert an ingredient, stroke with a knife,
    /// or cut on the sixth stroke.
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        click(host, event.click, event.click.stack)
    }

    /// A right click with an empty hand: a sneak takes the stack back.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        click(host, event.click, None)
    }
}

// ---------------------------------------------------------------------------
// The interaction
// ---------------------------------------------------------------------------

/// What one click has to know before it decides: the board's slot and strokes, and
/// whether the held item is a knife or an ingredient (`CuttingBoardTile.onActivated`).
struct Observed {
    slot: Option<ItemStack>,
    clicks: u32,
    recipe: Option<&'static Recipe>,
    knife: bool,
    ingredient: bool,
}

/// Read the board at `pos` and classify `held` against the recipe table and the knife
/// tags. `None` when there is no block entity there.
fn observe(
    host: &dyn Host,
    level: LevelRef,
    pos: [i32; 3],
    held: Option<&ItemStack>,
) -> Option<Observed> {
    let entity = host.block_entity(level, pos)?;
    let slot = entity
        .item_slot(host, INPUT_SLOT as u32)
        .ok()
        .flatten()
        .filter(|stack| stack.count() > 0);
    let clicks = saved_clicks(&entity.data(host).unwrap_or_default());
    let recipe = slot
        .as_ref()
        .and_then(|stack| recipe_for(&stack.item().id(host), |item, tag| in_tag(host, item, tag)));
    let (knife, ingredient) = match held.filter(|stack| stack.count() > 0) {
        Some(stack) => {
            let item = stack.item().id(host);
            (
                is_knife(&item, |item, tag| in_tag(host, item, tag)),
                accepts(&item, |item, tag| in_tag(host, item, tag)),
            )
        }
        None => (false, false),
    };
    Some(Observed {
        slot,
        clicks,
        recipe,
        knife,
        ingredient,
    })
}

/// Upstream's `onActivated`, in its order: a knife on a loaded board strokes it; an
/// ingredient on an empty board fills the slot; an empty hand while sneaking takes the
/// stack back; anything else falls through. `Some(Success)` replaces the block's own
/// behaviour, which is what the retired hook's `*canceled = true` meant.
fn click(
    host: &mut dyn ServerHost,
    click: &BlockClick<'_>,
    held: Option<&ItemStack>,
) -> Option<InteractionResult> {
    let level = host.level(click.dimension.key().id())?;
    let pos = click.pos;
    let board = observe(&*host, level, pos, held)?;
    if let Some(stack) = held.filter(|stack| stack.count() > 0) {
        // 1. a knife on a loaded board strokes it (`++click`).
        if board.slot.is_some() && board.knife {
            match stroke(board.clicks) {
                Stroke::Counting(clicks) => write_clicks(host, level, pos, clicks),
                Stroke::Cut => {
                    write_clicks(host, level, pos, 0);
                    if let Some(recipe) = board.recipe {
                        let slot = board.slot?;
                        cut(host, level, pos, recipe, slot, click.player);
                    }
                }
            }
            return Some(InteractionResult::Success);
        }
        // 2. an ingredient fills an empty board with as much of the held stack as fits
        //    (upstream inserts the whole stack and can lose the rest). The stack itself
        //    is moved, only its count reduced: a renamed or component-bearing ingredient
        //    keeps its identity on the board.
        if board.slot.is_none() && board.ingredient {
            let limit = host.max_stack_size(stack.item()).unwrap_or(MAX_STACK);
            let item = stack.item().id(&*host);
            let take = fits(None, &item, stack.count(), limit);
            if take > 0 && insert(host, level, pos, stack, take, click.hand, click.player) {
                return Some(InteractionResult::Success);
            }
        }
        return None;
    }

    // 3. an empty hand and a sneak takes the stack back (`player.isShiftKeyDown`).
    let player = host.player(click.player)?;
    if player.is_secondary_use_active(&*host) != Some(true) {
        return None;
    }
    let taken = board.slot?;
    take_back(host, level, pos, taken, player)?;
    Some(InteractionResult::Success)
}

/// Put `take` of `stack` on the board and take the same count out of the player's hand,
/// in one transaction: a creative player's hand gives without losing any, so there is no
/// separate creative check.
fn insert(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    stack: &ItemStack,
    take: u32,
    hand: Hand,
    player: u128,
) -> bool {
    let mut placed = stack.clone();
    if placed.set_count(take).is_err() {
        return false;
    }
    let resource = ItemResource::of(stack);
    let place = ItemPlace::PlayerHand { player, hand };
    host.transaction(|host, tx| {
        let entity = host.block_entity(level, pos).ok_or(())?;
        entity
            .set_item_slot(host, tx, INPUT_SLOT as u32, Some(placed.clone()))
            .map_err(|_| ())?;
        place.extract(host, tx, &resource, take);
        Ok::<(), ()>(())
    })
    .is_ok()
}

/// Hand the board's stack back: the slot is emptied and the whole stack given as `/give`
/// gives it (what does not fit is thrown for this player to pick up), in that order so a
/// failed give cannot duplicate it.
fn take_back(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    taken: ItemStack,
    player: PlayerRef,
) -> Option<()> {
    host.transaction(|host, tx| {
        let entity = host.block_entity(level, pos).ok_or(())?;
        entity
            .set_item_slot(host, tx, INPUT_SLOT as u32, None)
            .map_err(|_| ())?;
        Ok::<(), ()>(())
    })
    .ok()?;
    player.give_stack(host, taken).ok()?;
    Some(())
}

/// One cut: the recipe's `AmountItem` with the jar's amount, and the slot short one.
///
/// Upstream's amount is `min(maxCombine, extra + rand.nextInt(max - min) + min)`, with
/// `extra` from Steady Hands; this pin has no way to read a player's effects (a recorded
/// gap), so the roll alone is used — the same formula the retired port used.
fn cut(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    recipe: &'static Recipe,
    slot: ItemStack,
    player: u128,
) {
    let span = recipe.amount.max.saturating_sub(recipe.amount.min).max(1);
    let bound = i32::try_from(span).unwrap_or(i32::MAX);
    // Draw from the level's own stream and store it back, so a mod's draw moves vanilla's
    // later ones as a Java mod's does.
    let roll = level
        .with_random(host, |random| random.next_int_bound(bound).ok())
        .flatten()
        .map_or(0, |roll| u32::try_from(roll).unwrap_or(0));
    let amount = cut_amount(recipe.amount, roll);

    let left = slot.count().saturating_sub(1);
    let mut remaining = slot;
    let kept = (left > 0 && remaining.set_count(left).is_ok()).then_some(remaining);
    let _ = host.transaction(|host, tx| {
        let entity = host.block_entity(level, pos).ok_or(())?;
        entity
            .set_item_slot(host, tx, INPUT_SLOT as u32, kept.clone())
            .map_err(|_| ())?;
        Ok::<(), ()>(())
    });

    let Some(output) = output_stack(&*host, recipe, amount) else {
        return;
    };
    if let Some(player) = host.player(player) {
        let _ = player.give_stack(host, output);
    }
}

/// The recipe's output stack carrying its amount, through the component patch (the
/// registered `sushigocrafting:amount`, so the stack keeps it through saves and reaches
/// the client).
fn output_stack(host: &dyn Host, recipe: &'static Recipe, amount: u32) -> Option<ItemStack> {
    let mut output = ItemStack::new(item_key(host, recipe.output)?, 1)?;
    let mut patch = Compound::new();
    patch.insert(
        AMOUNT.into(),
        Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
    );
    output.apply_components(&patch).ok()?;
    Some(output)
}

/// A registry key for `id`, or `None` when this build has no such item.
fn item_key(host: &dyn Host, id: &str) -> Option<ItemKey> {
    let identifier = Identifier::parse(id).ok()?;
    host.item_key(&identifier)
}

/// Whether `item` (an id) is in the item tag `tag`, over the merged tags: what
/// `native_server_item_in_tag` read in the retired port. Both knife spellings are tested
/// here, so the upstream defect stays fixed (`notes.md`, "Upstream defects fixed").
fn in_tag(host: &dyn Host, item: &str, tag: &str) -> bool {
    let Ok(identifier) = Identifier::parse(tag) else {
        return false;
    };
    let Some(key) = host.tags().key(&identifier) else {
        return false;
    };
    let Some(item) = item_key(host, item) else {
        return false;
    };
    host.tags().is_in_tag(item, key).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Saved state
// ---------------------------------------------------------------------------

/// The board's stroke count from its saved compound (`CuttingBoardTile.click`), 0 for a
/// fresh board (upstream's `click` field starts at 0).
pub(crate) fn saved_clicks(nbt: &Compound) -> u32 {
    match nbt.get(&KEY_CLICKS.into()) {
        Some(Tag::Int(value)) => u32::try_from(*value).unwrap_or(0),
        _ => 0,
    }
}

/// Write the stroke count through `merge_data`, which marks the chunk and re-sends the
/// block entity, so a client's copy agrees (what `native_with_mod_block_entity` did).
fn write_clicks(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], clicks: u32) {
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    let mut patch = Compound::new();
    patch.insert(
        KEY_CLICKS.into(),
        Tag::Int(i32::try_from(clicks).unwrap_or(i32::MAX)),
    );
    let _ = entity.merge_data(host, patch);
}

// ---------------------------------------------------------------------------
// The `/sushigocrafting` commands
// ---------------------------------------------------------------------------

/// The commands (`register_commands`). Upstream has none: they exist so a player and the
/// harness can read the board without a GUI (precedent: `/naturesaura altar`,
/// `/industrialforegoing info`).
pub(crate) fn declare_commands(event: &mut RegisterCommandsEvent<'_>) {
    event
        .commands
        .push(Command::new("sushigocrafting board <pos:block_pos>"));
    event.commands.push(Command::op(
        "sushigocrafting clicks <pos:block_pos> <clicks:integer>",
        2,
    ));
}

/// The command bodies, on the typed invocation: `board` is the readout, `clicks` sets the
/// stroke count first (a harness control — `ctl interact` holds the button across two
/// ticks and can be heard twice, so a leg needs to stand at exactly five strokes).
pub(crate) fn run_command(host: &mut dyn ServerHost, invocation: &CommandInvocation) -> String {
    const USAGE: &str = "usage: /sushigocrafting board <x> <y> <z> | clicks <x> <y> <z> <n>";
    let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
        return USAGE.to_owned();
    };
    let pos = *pos;
    match invocation.literals.get(1).map(String::as_str) {
        Some("board") => readout(host, invocation, pos),
        Some("clicks") => {
            let Some(CommandArgumentValue::Integer(clicks)) = invocation.argument("clicks") else {
                return "usage: /sushigocrafting clicks <x> <y> <z> <n>".to_owned();
            };
            let clicks = u32::try_from(*clicks).unwrap_or(0);
            if let Some(level) = command_level(&*host, invocation) {
                write_clicks(host, level, pos, clicks);
            }
            readout(host, invocation, pos)
        }
        _ => USAGE.to_owned(),
    }
}

/// The readout line: `runtime::summary`, byte-identical to the retired port's, so the
/// existing in-game assertions still match.
fn readout(host: &mut dyn ServerHost, invocation: &CommandInvocation, pos: [i32; 3]) -> String {
    let Some(level) = command_level(&*host, invocation) else {
        return format!("{pos:?} is not a mod block entity.");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(String::new, |key| key.id(&*host));
    let Some(board) = observe(&*host, level, pos, None) else {
        return format!(
            "{block} at {} {} {}: no block entity",
            pos[0], pos[1], pos[2]
        );
    };
    let input = board
        .slot
        .as_ref()
        .map(|stack| (stack.item().id(&*host), stack.count()));
    summary(
        pos,
        input.as_ref().map(|(item, count)| (item.as_str(), *count)),
        board.clicks,
        board.recipe,
    )
}

/// The level a command executes in, from the dimension its invocation named (after
/// `execute in`).
fn command_level(host: &dyn Host, invocation: &CommandInvocation) -> Option<LevelRef> {
    invocation.dimension.id(host).and_then(|id| host.level(&id))
}

/// A feedback line to the command's source, vanilla's silence and track-output rules
/// included.
pub(crate) fn feedback(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_is_one_slot_reached_from_every_face_and_has_no_menu() {
        let container = container();
        assert_eq!(container.slots, SLOT_COUNT, "one slot");
        for face in container.faces {
            assert_eq!(face, vec![INPUT_SLOT], "every face reaches the one slot");
        }
        assert_eq!(container.inputs, vec![INPUT_SLOT], "automation may fill it");
        assert_eq!(
            container.outputs,
            vec![INPUT_SLOT],
            "automation may empty it"
        );
        assert_eq!(container.menu, None, "upstream has no GUI");
    }

    #[test]
    fn the_saved_stroke_count_reads_back_and_a_fresh_board_reads_zero() {
        let mut nbt = Compound::new();
        assert_eq!(saved_clicks(&nbt), 0, "a fresh board has not been stroked");
        nbt.insert(KEY_CLICKS.into(), Tag::Int(5));
        assert_eq!(saved_clicks(&nbt), 5);
        // A count saved as another type, or a negative one, is a board nobody stroked.
        nbt.insert(KEY_CLICKS.into(), Tag::String("3".into()));
        assert_eq!(saved_clicks(&nbt), 0);
        nbt.insert(KEY_CLICKS.into(), Tag::Int(-2));
        assert_eq!(saved_clicks(&nbt), 0);
    }

    #[test]
    fn the_block_and_its_type_are_the_same_namespaced_name() {
        assert_eq!(BOARD, "sushigocrafting:cutting_board");
        assert_eq!(
            BOARD_TYPE, "cutting_board",
            "`.blocks` adds the declaring mod's namespace"
        );
        assert_eq!(AMOUNT, "sushigocrafting:amount");
        assert_eq!(COMMAND, "sushigocrafting");
    }
}
