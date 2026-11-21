// The Sushi Go Crafting cutting board's rules, with no game access.
//
// Ported from Sushi Go Crafting 1.21-0.6.7 (`CuttingBoardTile`,
// `CuttingBoardRecipe`, `AmountItem`, `SushiContent.Items`); every constant
// below names the file it came from. This module is the standalone crate lib
// (`cargo test`) and is injected into the game as `crate::game::sushigocrafting`.

/// Knife strokes one cut takes. Upstream: `++click; if (click > 5)`, so the
/// sixth stroke cuts (`tile/machinery/CuttingBoardTile.java`).
pub const CLICKS_PER_CUT: u32 = 6;

/// The item tags that count as a knife: upstream's code tests `c:tools/knife`
/// and its own data ships `c:tools/knives` (see `FINDINGS.md`).
pub const KNIFE_TAGS: [&str; 2] = ["c:tools/knife", "c:tools/knives"];

/// A recipe input selector, as `recipe/cutting_board/*.json` writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// `"input": {"item": "minecraft:cod"}`.
    Id(&'static str),
    /// `"input": {"tag": "c:crops/cucumber"}`.
    Tag(&'static str),
}

/// An `AmountItem`'s bounds: `amountItem(id, category, min, max, combine, …)`
/// in `proxy/SushiContent.java`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amount {
    /// `minAmount`.
    pub min: u32,
    /// `maxAmount` (exclusive: `nextInt(max - min)`).
    pub max: u32,
    /// `maxCombineAmount`, which caps a single stack.
    pub max_combine: u32,
}

/// One cutting-board recipe: what it takes, what it gives, how much.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardRecipe {
    pub input: Input,
    pub output: &'static str,
    pub amount: Amount,
}

/// The jar's six recipes, in file order
/// (`data/sushigocrafting/recipe/cutting_board/`), with the ingredient each
/// name maps to (`api/impl/FoodIngredient.java`) and its amount bounds.
pub const BOARD_RECIPES: [BoardRecipe; 6] = [
    BoardRecipe {
        input: Input::Tag("c:fruits/avocado"),
        output: "sushigocrafting:avocado_slices",
        amount: Amount {
            min: 100,
            max: 500,
            max_combine: 1000,
        },
    },
    BoardRecipe {
        input: Input::Id("minecraft:cod"),
        output: "sushigocrafting:imitation_crab",
        amount: Amount {
            min: 50,
            max: 200,
            max_combine: 400,
        },
    },
    BoardRecipe {
        input: Input::Tag("c:crops/cucumber"),
        output: "sushigocrafting:cucumber_slices",
        amount: Amount {
            min: 50,
            max: 200,
            max_combine: 400,
        },
    },
    BoardRecipe {
        input: Input::Tag("c:raw_fishes/salmon"),
        output: "sushigocrafting:salmon_fillet",
        amount: Amount {
            min: 500,
            max: 2000,
            max_combine: 4000,
        },
    },
    BoardRecipe {
        input: Input::Tag("c:raw_fishes/tuna"),
        output: "sushigocrafting:tuna_fillet",
        amount: Amount {
            min: 1000,
            max: 3000,
            max_combine: 6000,
        },
    },
    BoardRecipe {
        input: Input::Tag("c:crops/wasabi_root"),
        output: "sushigocrafting:wasabi_paste",
        amount: Amount {
            min: 10,
            max: 50,
            max_combine: 100,
        },
    },
];

/// What one knife stroke does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stroke {
    /// The board counts the stroke; the value is the new stroke count.
    Counting(u32),
    /// The strokes are done: cut once and reset the count to zero.
    Cut,
}

/// The recipe whose input accepts `item`, given an item-tag lookup.
pub fn recipe_for(item: &str, in_tag: impl Fn(&str, &str) -> bool) -> Option<&'static BoardRecipe> {
    BOARD_RECIPES.iter().find(|recipe| match recipe.input {
        Input::Id(id) => id == item,
        Input::Tag(tag) => in_tag(item, tag),
    })
}

/// Whether the board's input filter takes `item`: some recipe's input matches.
pub fn accepts(item: &str, in_tag: impl Fn(&str, &str) -> bool) -> bool {
    recipe_for(item, in_tag).is_some()
}

/// Whether `item` may stroke the board (`CuttingBoardTile.KNIFE`).
pub fn is_knife(item: &str, in_tag: impl Fn(&str, &str) -> bool) -> bool {
    KNIFE_TAGS.iter().any(|tag| in_tag(item, tag))
}

/// One knife stroke, from the count of strokes already made.
pub fn stroke(clicks: u32) -> Stroke {
    let next = clicks.saturating_add(1);
    if next >= CLICKS_PER_CUT {
        Stroke::Cut
    } else {
        Stroke::Counting(next)
    }
}

/// The amount a cut gives: `min(max_combine, min + roll)`, where `roll` is
/// `nextInt(max - min)`. Steady Hands adds `(amplifier + 1) * min` upstream;
/// this pin has no way to read a player's effects (a recorded gap).
pub fn cut_amount(amount: Amount, roll: u32) -> u32 {
    let span = amount.max.saturating_sub(amount.min).max(1);
    let roll = roll.min(span - 1);
    amount.min.saturating_add(roll).min(amount.max_combine)
}

/// How many of `held` a single slot takes: none for another item, none when it
/// is full, otherwise what fits under `max`.
pub fn fits(slot: Option<(&str, u32)>, item: &str, held: u32, max: u32) -> u32 {
    let used = match slot {
        Some((slot_item, count)) if slot_item == item => count,
        Some(_) => return 0,
        None => 0,
    };
    max.saturating_sub(used).min(held)
}

/// The horizontal `subfacing` a placer with `yaw` (degrees, 0 south, 90 west)
/// gives the block: Titanium's `RotationType.FOUR_WAY` turns it toward its
/// placer, as `industrialforegoing` reads the same property.
pub fn facing_for_yaw(yaw: f32) -> &'static str {
    let quarter = ((f64::from(yaw) / 90.0 + 0.5).floor() as i64 & 3) as u8;
    match quarter {
        0 => "north",
        1 => "east",
        2 => "south",
        _ => "west",
    }
}

/// The `/sushigocrafting board` readout line, for players and for the harness.
pub fn summary(
    pos: [i32; 3],
    slot: Option<(&str, u32)>,
    clicks: u32,
    recipe: Option<&BoardRecipe>,
) -> String {
    let input = match slot {
        Some((item, count)) => format!("input={item} x{count}"),
        None => "input=none".to_owned(),
    };
    let next = match recipe {
        Some(recipe) => format!(
            "next={} {}..{}",
            recipe.output,
            recipe.amount.min,
            recipe.amount.max - 1
        ),
        None => "next=none".to_owned(),
    };
    format!(
        "sushigocrafting:cutting_board at {} {} {}: {input} clicks={clicks}/{CLICKS_PER_CUT} {next}",
        pos[0], pos[1], pos[2]
    )
}

// ---------------------------------------------------------------------------
// The fermentation barrel (`FermentationBarrelTile`, `FermentingBarrelRecipe`)
// ---------------------------------------------------------------------------

/// The barrel's tank, `FluidTankComponent("fluid", 2000, 59, 46)`.
pub const BARREL_TANK_MB: u32 = 2_000;
/// One batch's bar, `ProgressBarComponent(93, 48, 100)`.
pub const BARREL_TICKS: u32 = 100;
/// A bucket, vanilla's `FluidType.BUCKET_VOLUME`: what a bucket in hand moves
/// into or out of a tank (`FluidUtil.interactWithFluidHandler`).
pub const BUCKET_MB: u32 = 1_000;

/// `minecraft:milk`, the cheese recipe's fluid. NeoForge models it as a fluid;
/// see `notes.md` on why the port moves it itself.
pub const MILK: &str = "minecraft:milk";
/// The soy recipe's fluid.
pub const WATER: &str = "minecraft:water";

/// One `recipe/fermenting_barrel/*.json` row: an optional item input, the fluid
/// a batch drains, the item it gives and that item's amount bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarrelRecipe {
    /// `"input": {"tag": …}`, or `None` for `"input": []` — `Ingredient.EMPTY`,
    /// which `Ingredient.test` answers true only for an **empty** stack.
    pub input: Option<Input>,
    /// The fluid and the millibuckets one batch drains (`"fluid": {…}`).
    pub fluid: (&'static str, u32),
    pub output: &'static str,
    pub amount: Amount,
}

/// The jar's two recipes (`data/sushigocrafting/recipe/fermenting_barrel/`),
/// with the `amountItem(…)` bounds of the items they give
/// (`SushiContent.Items`: `cheese` 50/250/500, `soy_sauce` 10/50/100).
pub const BARREL_RECIPES: [BarrelRecipe; 2] = [
    BarrelRecipe {
        input: None,
        fluid: (MILK, 250),
        output: "sushigocrafting:cheese",
        amount: Amount {
            min: 50,
            max: 250,
            max_combine: 500,
        },
    },
    BarrelRecipe {
        input: Some(Input::Tag("c:crops/soybean")),
        fluid: (WATER, 250),
        output: "sushigocrafting:soy_sauce",
        amount: Amount {
            min: 10,
            max: 50,
            max_combine: 100,
        },
    },
];

/// Whether a recipe's item input takes the board's slot: `Ingredient.EMPTY`
/// (an `"input": []`) matches only an empty slot, an item or tag matches what
/// the item is in.
pub fn barrel_input_matches(
    recipe: &BarrelRecipe,
    input: Option<&str>,
    in_tag: impl Fn(&str, &str) -> bool,
) -> bool {
    match (recipe.input, input) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(_), None) => false,
        (Some(Input::Id(id)), Some(item)) => id == item,
        (Some(Input::Tag(tag)), Some(item)) => in_tag(item, tag),
    }
}

/// The recipe the barrel's tank and slot satisfy, in file order, as upstream's
/// `canStart` filters them: the input matches, the tank holds the recipe's
/// fluid with at least the recipe's millibuckets, and the output slot can take
/// one more.
pub fn barrel_recipe(
    input: Option<&str>,
    tank: Option<(&str, u32)>,
    in_tag: impl Fn(&str, &str) -> bool,
) -> Option<&'static BarrelRecipe> {
    BARREL_RECIPES.iter().find(|recipe| {
        barrel_input_matches(recipe, input, &in_tag)
            && tank.is_some_and(|(fluid, mb)| fluid == recipe.fluid.0 && mb >= recipe.fluid.1)
    })
}

/// Whether the barrel may start a batch: a recipe matches **and** its output
/// slot can take one more (`canStack`: empty, or the same item under
/// `maxCombineAmount`).
pub fn barrel_startable(
    input: Option<&str>,
    tank: Option<(&str, u32)>,
    output: Option<(&str, u32)>,
    in_tag: impl Fn(&str, &str) -> bool,
) -> Option<&'static BarrelRecipe> {
    let recipe = barrel_recipe(input, tank, in_tag)?;
    output_can_take(output, recipe).then_some(recipe)
}

/// `canStack`: an empty output slot, or the same item with room under its
/// `maxCombineAmount` (`AmountItem` stacks are one item whose amount grows).
pub fn output_can_take(output: Option<(&str, u32)>, recipe: &BarrelRecipe) -> bool {
    match output {
        None => true,
        Some((item, amount)) => item == recipe.output && amount < recipe.amount.max_combine,
    }
}

/// What one server tick of the bar does: count it, finish it, or reset it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Nothing matches: the bar resets to zero (`setCanReset`).
    Reset,
    /// The bar counts up; the value is the new progress.
    Counting(u32),
    /// The bar is full: one batch, then it starts again from zero.
    Finish,
}

/// One tick of `ProgressBarComponent`: up when the barrel can start, to
/// [`BARREL_TICKS`] when it finishes, and back to zero when it cannot.
pub fn barrel_step(progress: u32, startable: bool) -> Step {
    if !startable {
        return Step::Reset;
    }
    let next = progress.saturating_add(1);
    if next >= BARREL_TICKS {
        Step::Finish
    } else {
        Step::Counting(next)
    }
}

/// What one finished batch leaves behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Batch {
    /// The input slot's count after the batch; `None` when it empties.
    pub input_left: Option<u32>,
    /// The tank after the batch; `None` when it empties.
    pub fluid_left: Option<(&'static str, u32)>,
    /// The output slot's amount after the batch (it always holds one item).
    pub output_amount: u32,
}

/// `onFinish`: one batch of `recipe`, given the slots it read. The input
/// shrinks by one when the recipe has one, the tank drains the recipe's
/// millibuckets, and the output takes the rolled amount (upstream:
/// `AmountItem.random`, then `combineStacks` into a slot already holding one).
pub fn barrel_finish(
    recipe: &BarrelRecipe,
    input: Option<(&str, u32)>,
    tank: Option<(&str, u32)>,
    output: Option<(&str, u32)>,
    roll: u32,
) -> Batch {
    let input_left = match (recipe.input, input) {
        (Some(_), Some((_, count))) => count.checked_sub(1).filter(|left| *left > 0),
        (None, Some((_, count))) => Some(count),
        (_, None) => None,
    };
    let fluid_left = tank.and_then(|(_, mb)| {
        mb.checked_sub(recipe.fluid.1)
            .filter(|left| *left > 0)
            .map(|left| (recipe.fluid.0, left))
    });
    let rolled = cut_amount(recipe.amount, roll);
    Batch {
        input_left,
        fluid_left,
        output_amount: combined_amount(output, recipe.output, rolled, recipe.amount.max_combine),
    }
}

/// The output slot's amount after a batch: the roll alone into an empty or
/// different slot, `min(maxCombine, have + roll)` into one holding the same
/// item (`AmountItem.combineStacks`).
pub fn combined_amount(
    output: Option<(&str, u32)>,
    item: &str,
    rolled: u32,
    max_combine: u32,
) -> u32 {
    match output {
        Some((have_item, have)) if have_item == item => {
            have.saturating_add(rolled).min(max_combine)
        }
        _ => rolled.min(max_combine),
    }
}

/// What a right click with a container in hand does to the barrel's tank. The
/// barrel's tank is `FluidTankComponent.Action.FILL`: a container in hand fills
/// it, and it never drains into one — the machine drains itself on a batch
/// (`drainForced`), so a player gets no fluid back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket {
    /// Put a bucket of this fluid in: the hand keeps the empty bucket.
    Fill { fluid: &'static str },
    /// A bucket the tank cannot take, or an empty bucket the tank never gives to.
    Refused,
    /// Not a bucket: another fluid container, or nothing.
    NotYours,
}

/// `minecraft:lava`, which no recipe needs but the tank still takes.
const LAVA: &str = "minecraft:lava";
/// The full buckets and the fluid each holds.
const FULL_BUCKETS: [(&str, &str); 3] = [
    ("minecraft:water_bucket", WATER),
    ("minecraft:lava_bucket", LAVA),
    ("minecraft:milk_bucket", MILK),
];

/// A click with `held` on a tank holding `tank` (fluid, millibuckets), for the
/// barrel's 2,000 mB tank. A full bucket fills it when the tank has room and is
/// empty or holds the same fluid; an empty bucket is refused.
pub fn bucket_click(held: &str, tank: Option<(&str, u32)>) -> Bucket {
    let Some((_, fluid)) = FULL_BUCKETS.iter().find(|(item, _)| *item == held) else {
        return if held == "minecraft:bucket" {
            Bucket::Refused
        } else {
            Bucket::NotYours
        };
    };
    let room = match tank {
        Some((have, mb)) if have == *fluid => BARREL_TANK_MB.saturating_sub(mb),
        Some(_) => 0,
        None => BARREL_TANK_MB,
    };
    if room >= BUCKET_MB {
        Bucket::Fill { fluid }
    } else {
        Bucket::Refused
    }
}

/// The `/sushigocrafting barrel` readout line, for players and for the harness.
pub fn barrel_summary(
    pos: [i32; 3],
    input: Option<(&str, u32)>,
    tank: Option<(&str, u32)>,
    progress: u32,
    next: Option<&BarrelRecipe>,
) -> String {
    let input = match input {
        Some((item, count)) => format!("input={item} x{count}"),
        None => "input=none".to_owned(),
    };
    let fluid = match tank {
        Some((fluid, mb)) => format!("fluid={fluid} {mb}/{BARREL_TANK_MB} mB"),
        None => format!("fluid=none 0/{BARREL_TANK_MB} mB"),
    };
    let next = match next {
        Some(recipe) => format!(
            "next={} {}..{}",
            recipe.output,
            recipe.amount.min,
            recipe.amount.max - 1
        ),
        None => "next=none".to_owned(),
    };
    format!(
        "sushigocrafting:fermentation_barrel at {} {} {}: {input} {fluid} progress={progress}/{BARREL_TICKS} {next}",
        pos[0], pos[1], pos[2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The jar's tags, as `data/c/tags/item/**` writes them.
    fn tags(item: &str, tag: &str) -> bool {
        match tag {
            "c:crops/cucumber" => item == "sushigocrafting:cucumber",
            "c:fruits/avocado" => item == "sushigocrafting:avocado",
            "c:crops/wasabi_root" => item == "sushigocrafting:wasabi_root",
            "c:raw_fishes/tuna" => item == "sushigocrafting:raw_tuna",
            "c:crops/soybean" => item == "sushigocrafting:soy_bean",
            "c:tools/knives" => item == "sushigocrafting:cleaver_knife",
            _ => false,
        }
    }

    #[test]
    fn the_six_recipes_are_the_jars() {
        let want = [
            (
                Input::Tag("c:fruits/avocado"),
                "sushigocrafting:avocado_slices",
                (100, 500, 1000),
            ),
            (
                Input::Id("minecraft:cod"),
                "sushigocrafting:imitation_crab",
                (50, 200, 400),
            ),
            (
                Input::Tag("c:crops/cucumber"),
                "sushigocrafting:cucumber_slices",
                (50, 200, 400),
            ),
            (
                Input::Tag("c:raw_fishes/salmon"),
                "sushigocrafting:salmon_fillet",
                (500, 2000, 4000),
            ),
            (
                Input::Tag("c:raw_fishes/tuna"),
                "sushigocrafting:tuna_fillet",
                (1000, 3000, 6000),
            ),
            (
                Input::Tag("c:crops/wasabi_root"),
                "sushigocrafting:wasabi_paste",
                (10, 50, 100),
            ),
        ];
        assert_eq!(BOARD_RECIPES.len(), want.len(), "the jar has six recipes");
        for (recipe, (input, output, (min, max, combine))) in BOARD_RECIPES.iter().zip(want) {
            assert_eq!(recipe.input, input, "{output} input");
            assert_eq!(recipe.output, output, "{input:?} output");
            assert_eq!(
                recipe.amount,
                Amount {
                    min,
                    max,
                    max_combine: combine
                },
                "{output} amount"
            );
        }
    }

    #[test]
    fn an_item_and_a_tag_both_find_their_recipe() {
        let cod = recipe_for("minecraft:cod", tags).expect("cod is a recipe input");
        assert_eq!(cod.output, "sushigocrafting:imitation_crab");
        let tuna =
            recipe_for("sushigocrafting:raw_tuna", tags).expect("the tuna tag is a recipe input");
        assert_eq!(tuna.output, "sushigocrafting:tuna_fillet");
    }

    #[test]
    fn a_non_ingredient_is_not_a_recipe_input() {
        assert!(recipe_for("minecraft:dirt", tags).is_none());
        assert!(!accepts("minecraft:stick", tags));
        assert!(accepts("sushigocrafting:cucumber", tags));
    }

    #[test]
    fn the_jars_cleaver_is_a_knife_and_a_stick_is_not() {
        assert!(is_knife("sushigocrafting:cleaver_knife", tags));
        assert!(is_knife("farmersdelight:iron_knife", |item, tag| {
            tag == "c:tools/knife" && item == "farmersdelight:iron_knife"
        }));
        assert!(!is_knife("minecraft:stick", tags));
    }

    #[test]
    fn five_strokes_count_and_the_sixth_cuts() {
        assert_eq!(stroke(0), Stroke::Counting(1));
        assert_eq!(stroke(1), Stroke::Counting(2));
        assert_eq!(stroke(4), Stroke::Counting(5));
        assert_eq!(stroke(5), Stroke::Cut);
    }

    #[test]
    fn a_cut_gives_the_jar_amount_for_the_roll() {
        let cucumber = BOARD_RECIPES[2].amount;
        assert_eq!(cut_amount(cucumber, 0), 50);
        assert_eq!(cut_amount(cucumber, 149), 199);
        assert_eq!(cut_amount(cucumber, 99_999), 199);
        let tuna = BOARD_RECIPES[4].amount;
        assert_eq!(cut_amount(tuna, 0), 1000);
        assert_eq!(cut_amount(tuna, 1999), 2999);
    }

    #[test]
    fn a_single_stack_stops_at_max_combine() {
        let capped = Amount {
            min: 100,
            max: 200,
            max_combine: 150,
        };
        assert_eq!(cut_amount(capped, 60), 150);
    }

    #[test]
    fn a_slot_takes_only_its_own_item_and_only_what_fits() {
        assert_eq!(fits(None, "sushigocrafting:cucumber", 5, 64), 5);
        assert_eq!(
            fits(
                Some(("sushigocrafting:cucumber", 60)),
                "sushigocrafting:cucumber",
                64,
                64
            ),
            4
        );
        assert_eq!(
            fits(
                Some(("sushigocrafting:cucumber", 10)),
                "minecraft:cod",
                1,
                64
            ),
            0
        );
        assert_eq!(
            fits(
                Some(("sushigocrafting:cucumber", 64)),
                "sushigocrafting:cucumber",
                3,
                64
            ),
            0
        );
    }

    #[test]
    fn the_board_turns_toward_its_placer() {
        assert_eq!(facing_for_yaw(0.0), "north");
        assert_eq!(facing_for_yaw(90.0), "east");
        assert_eq!(facing_for_yaw(180.0), "south");
        assert_eq!(facing_for_yaw(-90.0), "west");
        assert_eq!(facing_for_yaw(350.0), "north");
    }

    #[test]
    fn the_readout_names_the_input_the_strokes_and_the_next_output() {
        let line = summary(
            [202, 101, 0],
            Some(("sushigocrafting:cucumber", 1)),
            3,
            Some(&BOARD_RECIPES[2]),
        );
        assert!(line.contains("at 202 101 0"), "{line}");
        assert!(line.contains("input=sushigocrafting:cucumber x1"), "{line}");
        assert!(line.contains("clicks=3/6"), "{line}");
        assert!(
            line.contains("next=sushigocrafting:cucumber_slices 50..199"),
            "{line}"
        );
        let empty = summary([1, 2, 3], None, 0, None);
        assert!(empty.contains("input=none"), "{empty}");
        assert!(empty.contains("clicks=0/6"), "{empty}");
        assert!(empty.contains("next=none"), "{empty}");
    }

    // -----------------------------------------------------------------------
    // The fermentation barrel
    // -----------------------------------------------------------------------

    #[test]
    fn the_two_barrel_recipes_are_the_jars() {
        // `data/sushigocrafting/recipe/fermenting_barrel/cheese.json` has
        // `"input": []` and milk 250; `soy.json` takes `#c:crops/soybean` and
        // water 250. The amounts are `amountItem("cheese", …, 50, 250, 500)` and
        // `amountItem("soy_sauce", …, 10, 50, 100)` in `SushiContent.Items`.
        assert_eq!(BARREL_RECIPES.len(), 2, "the jar has two barrel recipes");
        assert_eq!(
            BARREL_RECIPES[0],
            BarrelRecipe {
                input: None,
                fluid: (MILK, 250),
                output: "sushigocrafting:cheese",
                amount: Amount {
                    min: 50,
                    max: 250,
                    max_combine: 500,
                },
            },
            "cheese"
        );
        assert_eq!(
            BARREL_RECIPES[1],
            BarrelRecipe {
                input: Some(Input::Tag("c:crops/soybean")),
                fluid: (WATER, 250),
                output: "sushigocrafting:soy_sauce",
                amount: Amount {
                    min: 10,
                    max: 50,
                    max_combine: 100,
                },
            },
            "soy sauce"
        );
        assert_eq!(BARREL_TANK_MB, 2_000, "the jar's tank");
        assert_eq!(BARREL_TICKS, 100, "the jar's bar");
    }

    #[test]
    fn an_empty_slot_is_the_cheese_recipe_and_a_soy_bean_is_the_soy_one() {
        // `Ingredient.EMPTY.test(empty)` is true, so cheese runs with nothing in
        // the input slot; anything in it rules cheese out.
        assert_eq!(
            barrel_recipe(None, Some((MILK, 250)), tags).map(|r| r.output),
            Some("sushigocrafting:cheese")
        );
        assert!(
            barrel_recipe(Some("sushigocrafting:soy_bean"), Some((MILK, 250)), tags).is_none(),
            "a filled slot is not the cheese recipe"
        );
        assert_eq!(
            barrel_recipe(Some("sushigocrafting:soy_bean"), Some((WATER, 250)), tags)
                .map(|r| r.output),
            Some("sushigocrafting:soy_sauce")
        );
        assert!(
            barrel_recipe(None, Some((WATER, 250)), tags).is_none(),
            "soy sauce needs its bean"
        );
        assert!(
            barrel_recipe(Some("sushigocrafting:cucumber"), Some((WATER, 2_000)), tags).is_none(),
            "a cucumber is not the soy recipe's input"
        );
    }

    #[test]
    fn the_tank_must_hold_the_recipes_own_fluid_and_enough_of_it() {
        let bean = Some("sushigocrafting:soy_bean");
        assert!(
            barrel_recipe(bean, Some((WATER, 249)), tags).is_none(),
            "249 mB is short of the 250 a batch drains"
        );
        assert!(barrel_recipe(bean, Some((WATER, 250)), tags).is_some());
        assert!(
            barrel_recipe(bean, None, tags).is_none(),
            "an empty tank starts nothing"
        );
        assert!(
            barrel_recipe(bean, Some((MILK, 2_000)), tags).is_none(),
            "milk is not the soy recipe's fluid"
        );
    }

    #[test]
    fn a_full_or_foreign_output_slot_starts_nothing() {
        let cheese = |slot| barrel_startable(None, Some((MILK, 1_000)), slot, tags);
        assert!(cheese(None).is_some(), "an empty output slot takes one");
        assert!(
            cheese(Some(("sushigocrafting:cheese", 499))).is_some(),
            "one under maxCombineAmount is room"
        );
        assert!(
            cheese(Some(("sushigocrafting:cheese", 500))).is_none(),
            "500 is the cap"
        );
        assert!(
            cheese(Some(("minecraft:dirt", 1))).is_none(),
            "another item is not the recipe's output"
        );
    }

    #[test]
    fn ninety_nine_ticks_count_and_the_hundredth_finishes() {
        assert_eq!(barrel_step(0, true), Step::Counting(1));
        assert_eq!(barrel_step(98, true), Step::Counting(99));
        assert_eq!(barrel_step(99, true), Step::Finish);
        // A barrel that cannot start resets, as `setCanReset` does.
        assert_eq!(barrel_step(37, false), Step::Reset);
        assert_eq!(barrel_step(0, false), Step::Reset);
    }

    #[test]
    fn finishing_takes_one_input_drains_a_batch_and_rolls_the_output() {
        let soy = &BARREL_RECIPES[1];
        let batch = barrel_finish(
            soy,
            Some(("sushigocrafting:soy_bean", 3)),
            Some((WATER, 2_000)),
            None,
            17,
        );
        assert_eq!(batch.input_left, Some(2), "one bean per batch");
        assert_eq!(batch.fluid_left, Some((WATER, 1_750)), "250 mB a batch");
        assert_eq!(batch.output_amount, 27, "10 + 17, inside 10..49");
        // The last bean empties the slot; the last bucket empties the tank.
        let last = barrel_finish(
            soy,
            Some(("sushigocrafting:soy_bean", 1)),
            Some((WATER, 250)),
            None,
            0,
        );
        assert_eq!(last.input_left, None);
        assert_eq!(last.fluid_left, None);
        // Cheese runs with an empty input slot, which stays empty.
        let cheese = barrel_finish(&BARREL_RECIPES[0], None, Some((MILK, 2_000)), None, 0);
        assert_eq!(cheese.input_left, None);
        assert_eq!(cheese.fluid_left, Some((MILK, 1_750)));
        assert_eq!(cheese.output_amount, 50);
    }

    #[test]
    fn an_output_already_holding_the_item_adds_the_roll_up_to_its_cap() {
        let soy = &BARREL_RECIPES[1];
        // `AmountItem.combineStacks`: min(maxCombine, have + rolled).
        assert_eq!(
            combined_amount(Some(("sushigocrafting:soy_sauce", 90)), soy.output, 30, 100),
            100
        );
        assert_eq!(
            combined_amount(Some(("sushigocrafting:soy_sauce", 40)), soy.output, 30, 100),
            70
        );
        assert_eq!(
            combined_amount(Some(("minecraft:dirt", 40)), soy.output, 30, 100),
            30,
            "another item's slot is replaced, not added to"
        );
    }

    #[test]
    fn a_bucket_fills_the_tank_and_an_empty_bucket_is_refused() {
        assert_eq!(
            bucket_click("minecraft:milk_bucket", None),
            Bucket::Fill { fluid: MILK }
        );
        assert_eq!(
            bucket_click("minecraft:water_bucket", Some((WATER, 1_000))),
            Bucket::Fill { fluid: WATER }
        );
        assert_eq!(
            bucket_click("minecraft:water_bucket", Some((MILK, 1_000))),
            Bucket::Refused,
            "a tank holding milk takes no water"
        );
        assert_eq!(
            bucket_click("minecraft:milk_bucket", Some((MILK, 1_500))),
            Bucket::Refused,
            "500 mB of room is not a bucket"
        );
        // `Action.FILL`: the tank never gives a player its fluid back.
        assert_eq!(
            bucket_click("minecraft:bucket", Some((WATER, 1_000))),
            Bucket::Refused
        );
        assert_eq!(bucket_click("minecraft:bucket", None), Bucket::Refused);
        assert_eq!(
            bucket_click("minecraft:stick", Some((WATER, 1_000))),
            Bucket::NotYours
        );
    }

    #[test]
    fn the_barrel_readout_names_the_input_the_tank_the_bar_and_the_next_output() {
        let line = barrel_summary(
            [203, 101, 0],
            Some(("sushigocrafting:soy_bean", 2)),
            Some((WATER, 1_000)),
            41,
            Some(&BARREL_RECIPES[1]),
        );
        assert!(line.contains("at 203 101 0"), "{line}");
        assert!(line.contains("input=sushigocrafting:soy_bean x2"), "{line}");
        assert!(
            line.contains("fluid=minecraft:water 1000/2000 mB"),
            "{line}"
        );
        assert!(line.contains("progress=41/100"), "{line}");
        assert!(
            line.contains("next=sushigocrafting:soy_sauce 10..49"),
            "{line}"
        );
        let empty = barrel_summary([0, 0, 0], None, None, 0, None);
        assert!(empty.contains("input=none"), "{empty}");
        assert!(empty.contains("fluid=none 0/2000 mB"), "{empty}");
        assert!(empty.contains("progress=0/100"), "{empty}");
        assert!(empty.contains("next=none"), "{empty}");
    }
}
