// Game-side glue for the Sushi Go Crafting cutting board (slice 1).
//
// Injected into `crate::game` as `sushigocrafting_state`; it needs the game, so
// it is not part of the standalone library (only `runtime.rs` is).
//
// The board's block entity keeps one item slot and the knife-stroke count in its
// saved compound. A right click inserts a recipe's ingredient into an empty
// board, strokes it with a knife (`c:tools/knife` or the jar's own
// `c:tools/knives`), cuts on the sixth stroke and hands the player the recipe's
// AmountItem, or hands the stack back on a sneak with an empty hand. Read from
// Sushi Go Crafting 1.21-0.6.7 `CuttingBoardTile.onActivated` (FINDINGS.md).

use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_core::random::RandomSource;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;

use crate::game::Game;
use crate::game::native_server::{NativeCommand, NativeServerPlayer};
use crate::game::sushigocrafting::{
    Recipe, Stroke, accepts, cut_amount, facing_for_yaw, fits, is_knife, recipe_for, stroke,
    summary,
};

/// The block this bundle is about.
pub const BOARD: &str = "sushigocrafting:cutting_board";
/// The state property Titanium's `RotationType.FOUR_WAY` uses, which the jar's
/// own blockstate selects on.
pub const SUBFACING: &str = "subfacing";
/// The block entity's saved stroke count (`CuttingBoardTile.click`, `@Save`).
const KEY_CLICKS: &str = "clicks";
/// The one container slot and its limit (`InventoryComponent("input", 0, 0, 1)`,
/// whose default slot limit is a stack of 64).
const INPUT_SLOT: usize = 0;
const SLOT_COUNT: usize = 1;
const MAX_STACK: u32 = 64;
/// The component an AmountItem carries (`SushiDataComponent.AMOUNT`); Horizon
/// keeps components it does not model as their saved values (`StackData::other`).
const AMOUNT_COMPONENT: &str = "sushigocrafting:amount";

// ---------------------------------------------------------------------------
// Declaration hooks
// ---------------------------------------------------------------------------

/// Registers the AmountItem's component (`native_component_types`, API 65),
/// before the content mod's items load.
pub fn register_component_types(types: &mut Vec<String>) {
    types.push(AMOUNT_COMPONENT.to_owned());
}

/// Give the board a block entity (`native_block_entity`, API 15).
#[allow(private_interfaces)]
pub fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if block == BOARD {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// One slot, every face, no menu: upstream has a `InventoryComponent` and no
/// GUI (`CuttingBoardTile` never calls `openGui`). The slot's input filter is
/// the right-click path's (a pending `container_accepts` hook would refuse a
/// hopper's wrong item; recorded in `notes.md`).
#[allow(private_interfaces)]
pub fn on_container(block: &str, container: &mut Option<ModContainer>) {
    if block != BOARD {
        return;
    }
    *container = Some(ModContainer {
        slots: SLOT_COUNT,
        faces: std::array::from_fn(|_| vec![INPUT_SLOT]),
        inputs: vec![INPUT_SLOT],
        outputs: vec![INPUT_SLOT],
        menu: None,
    });
}

/// The board faces its placer (`Titanium`'s `FOUR_WAY`), as the industrialforegoing
/// port does for the same property (`native_state_for_placement`, API 76).
#[allow(private_interfaces)]
pub fn on_state_for_placement(
    level: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    context: &horizon_world::level::PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    let Some(name) = level.registry.state(state).map(|state| state.name.clone()) else {
        return;
    };
    if &*name != BOARD {
        return;
    }
    if let Some(faced) =
        level.native_state_with_property(state, SUBFACING, facing_for_yaw(context.yaw))
    {
        *answer = Some(Some(faced));
    }
}

// ---------------------------------------------------------------------------
// The interaction
// ---------------------------------------------------------------------------

/// Right click (`native_right_click_block`, API 21): insert an ingredient, stroke
/// with a knife, cut on the sixth stroke, or hand the stack back while sneaking.
#[allow(private_interfaces)]
pub fn on_right_click(game: &mut Game<'_>, pos: [i32; 3], _face: [i32; 3], canceled: &mut bool) {
    if game.native_block(pos) != BOARD {
        return;
    }
    let Some(board) = read_board(game, pos) else {
        return;
    };
    let held = game.native_held().filter(|(_, count, _)| *count > 0);
    let sneaking = game
        .native_acting_player()
        .is_some_and(|player| player.sneaking);
    let recipe = board
        .item
        .as_ref()
        .and_then(|(item, _)| recipe_for(item, |test, tag| in_tag(game, test, tag)));
    let knife = held
        .as_ref()
        .is_some_and(|(item, _, _)| is_knife(item, |test, tag| in_tag(game, test, tag)));
    let ingredient = held
        .as_ref()
        .is_some_and(|(item, _, _)| accepts(item, |test, tag| in_tag(game, test, tag)));

    if let Some((item, count, _)) = held {
        // 1. a knife on a loaded board strokes it (`++click`).
        if board.item.is_some() && knife {
            match stroke(board.clicks) {
                Stroke::Counting(clicks) => write_clicks(game, pos, clicks),
                Stroke::Cut => {
                    write_clicks(game, pos, 0);
                    if let Some(recipe) = recipe {
                        cut(game, pos, &board, recipe);
                    }
                }
            }
            *canceled = true;
            return;
        }
        // 2. an ingredient fills an empty board with as much of the held stack
        //    as fits (upstream inserts the whole stack and can lose the rest).
        if board.item.is_none() && ingredient {
            let fits = fits(None, &item, count, MAX_STACK);
            if fits > 0 {
                write_slot(game, pos, Some(stack(&item, fits)));
                game.native_take_items(&item, fits);
                *canceled = true;
            }
            return;
        }
        return;
    }

    // 3. an empty hand and sneak takes the stack back (`player.isShiftKeyDown`).
    if sneaking && board.item.is_some() {
        let taken = take_slot(game, pos);
        if let Some(taken) = taken {
            give(game, pos, taken);
        }
        *canceled = true;
    }
}

/// One cut: the recipe's AmountItem with the jar's amount, the slot short one.
fn cut(game: &mut Game<'_>, pos: [i32; 3], board: &Board, recipe: &'static Recipe) {
    let Some((item, count)) = board.item.clone() else {
        return;
    };
    let span = recipe.amount.max.saturating_sub(recipe.amount.min).max(1);
    let bound = i32::try_from(span).unwrap_or(i32::MAX);
    let dimension = game.native_dimension().to_owned();
    let roll = game
        .native_level_random(&dimension)
        .and_then(|random| random.next_int_bound(bound).ok())
        .map_or(0, |roll| u32::try_from(roll).unwrap_or(0));
    let amount = cut_amount(recipe.amount, roll);
    let left = count.saturating_sub(1);
    write_slot(
        game,
        pos,
        if left > 0 {
            Some(stack(&item, left))
        } else {
            None
        },
    );
    let mut output = ItemStack {
        item: recipe.output.into(),
        count: 1,
        damage: 0,
        extra: None,
    };
    // `AmountItem.withAmount`: the stack's own `sushigocrafting:amount`, through
    // the hook that knows the registered type (API 65) so it reaches the client.
    if game
        .native_set_component(&mut output, AMOUNT_COMPONENT, Tag::Int(amount as i32))
        .is_err()
    {
        log::warn!("Horizon: the cut's amount component could not be set");
    }
    give(game, pos, output);
}

/// Gives `output` to the acting player, or drops it at the board when it does
/// not fit (upstream gives it to the player or nowhere).
fn give(game: &mut Game<'_>, pos: [i32; 3], output: ItemStack) {
    let left = game
        .native_acting_player()
        .and_then(|player| game.native_give_stack(player.uuid, output.clone()))
        .filter(|left| left.count > 0);
    if let Some(left) = left {
        game.pop_resource(pos, left);
    }
}

// ---------------------------------------------------------------------------
// The `/sushigocrafting board` readout
// ---------------------------------------------------------------------------

/// The commands (`native_register_commands`, API 8). Upstream has none: this one
/// exists so a player and the harness can read the board without a GUI.
#[allow(private_interfaces)]
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("sushigocrafting board"));
    commands.push(NativeCommand::new("sushigocrafting clicks"));
}

/// `/sushigocrafting board <x> <y> <z>` (`native_perform_command`, API 8).
#[allow(private_interfaces)]
pub fn on_perform_command(
    game: &mut Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() != Some(&"sushigocrafting") {
        return;
    }
    *ran = true;
    let numbers = (2..)
        .take_while(|index| words.get(*index).is_some())
        .map(|index| words.get(index).and_then(|word| word.parse::<i32>().ok()))
        .collect::<Option<Vec<i32>>>();
    let Some(numbers) = numbers.filter(|numbers| numbers.len() >= 3) else {
        game.native_feedback(Text::literal(
            "usage: /sushigocrafting board <x> <y> <z> | clicks <x> <y> <z> <n>",
        ));
        return;
    };
    let pos = [numbers[0], numbers[1], numbers[2]];
    // `/sushigocrafting clicks <x> <y> <z> <n>`: the harness sets the stroke
    // count, as industrialforegoing's `/industrialforegoing progress` sets its
    // bar, so a leg can stand at five strokes without seven clicks (`ctl
    // interact` holds the button for two ticks and can fire twice).
    if words.get(1) == Some(&"clicks") && numbers.len() >= 4 {
        write_clicks(game, pos, u32::try_from(numbers[3]).unwrap_or(0));
    }
    let block = game.native_block(pos);
    let line = match read_board(game, pos) {
        Some(board) => {
            let recipe = board
                .item
                .as_ref()
                .and_then(|(item, _)| recipe_for(item, |test, tag| in_tag(game, test, tag)));
            summary(
                pos,
                board.item.as_ref().map(|(item, count)| (&**item, *count)),
                board.clicks,
                recipe,
            )
        }
        None => format!(
            "{block} at {} {} {}: no block entity",
            pos[0], pos[1], pos[2]
        ),
    };
    game.native_feedback(Text::literal(line));
}

// ---------------------------------------------------------------------------
// Shared reads and writes
// ---------------------------------------------------------------------------

/// What the board's block entity holds right now.
struct Board {
    item: Option<(String, u32)>,
    clicks: u32,
}

fn read_board(game: &Game<'_>, pos: [i32; 3]) -> Option<Board> {
    let entity = match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(entity)) if entity.id == BOARD => entity,
        _ => return None,
    };
    let item = entity
        .items
        .get(INPUT_SLOT)
        .and_then(|slot| slot.as_ref())
        .filter(|slot| slot.count > 0)
        .map(|slot| (slot.item.to_string(), slot.count));
    Some(Board {
        item,
        clicks: nbt_u32(&entity.nbt, KEY_CLICKS),
    })
}

/// Writes one slot through API 18's helper, which marks the chunk and tells
/// comparators when the items changed.
fn write_slot(game: &mut Game<'_>, pos: [i32; 3], slot: Option<ItemStack>) {
    let _ = game.native_with_mod_block_entity(pos, |entity: &mut ModBlockEntity| {
        if entity.id != BOARD {
            return;
        }
        if entity.items.len() < SLOT_COUNT {
            entity.items.resize(SLOT_COUNT, None);
        }
        entity.items[INPUT_SLOT] = slot;
    });
}

fn take_slot(game: &mut Game<'_>, pos: [i32; 3]) -> Option<ItemStack> {
    game.native_with_mod_block_entity(pos, |entity: &mut ModBlockEntity| {
        if entity.id != BOARD {
            return None;
        }
        let taken = entity.items.get_mut(INPUT_SLOT).and_then(Option::take);
        taken.filter(|slot| slot.count > 0)
    })
    .flatten()
}

fn write_clicks(game: &mut Game<'_>, pos: [i32; 3], clicks: u32) {
    let _ = game.native_with_mod_block_entity(pos, |entity: &mut ModBlockEntity| {
        if entity.id != BOARD {
            return;
        }
        entity
            .nbt
            .insert(JavaString::from(KEY_CLICKS), Tag::Int(clicks as i32));
    });
}

fn stack(item: &str, count: u32) -> ItemStack {
    ItemStack {
        item: item.into(),
        count,
        damage: 0,
        extra: None,
    }
}

/// Whether `item` is in the server's merged item tag `tag` (`native_server_item_in_tag`,
/// API 63: the tags this bundle's content carries, plus every other mod's).
fn in_tag(game: &Game<'_>, item: &str, tag: &str) -> bool {
    game.native_server_item_in_tag(&stack(item, 1), tag)
        .unwrap_or(false)
}

fn nbt_u32(tag: &Compound, key: &str) -> u32 {
    match tag.get(&JavaString::from(key)) {
        Some(Tag::Int(value)) => u32::try_from(*value).unwrap_or(0),
        _ => 0,
    }
}
