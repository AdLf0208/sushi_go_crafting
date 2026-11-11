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
    Item(&'static str),
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
pub struct Recipe {
    pub input: Input,
    pub output: &'static str,
    pub amount: Amount,
}

/// The jar's six recipes, in file order
/// (`data/sushigocrafting/recipe/cutting_board/`), with the ingredient each
/// name maps to (`api/impl/FoodIngredient.java`) and its amount bounds.
pub const RECIPES: [Recipe; 6] = [
    Recipe {
        input: Input::Tag("c:fruits/avocado"),
        output: "sushigocrafting:avocado_slices",
        amount: Amount {
            min: 100,
            max: 500,
            max_combine: 1000,
        },
    },
    Recipe {
        input: Input::Item("minecraft:cod"),
        output: "sushigocrafting:imitation_crab",
        amount: Amount {
            min: 50,
            max: 200,
            max_combine: 400,
        },
    },
    Recipe {
        input: Input::Tag("c:crops/cucumber"),
        output: "sushigocrafting:cucumber_slices",
        amount: Amount {
            min: 50,
            max: 200,
            max_combine: 400,
        },
    },
    Recipe {
        input: Input::Tag("c:raw_fishes/salmon"),
        output: "sushigocrafting:salmon_fillet",
        amount: Amount {
            min: 500,
            max: 2000,
            max_combine: 4000,
        },
    },
    Recipe {
        input: Input::Tag("c:raw_fishes/tuna"),
        output: "sushigocrafting:tuna_fillet",
        amount: Amount {
            min: 1000,
            max: 3000,
            max_combine: 6000,
        },
    },
    Recipe {
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
pub fn recipe_for(item: &str, in_tag: impl Fn(&str, &str) -> bool) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| match recipe.input {
        Input::Item(id) => id == item,
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
    recipe: Option<&Recipe>,
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
                Input::Item("minecraft:cod"),
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
        assert_eq!(RECIPES.len(), want.len(), "the jar has six recipes");
        for (recipe, (input, output, (min, max, combine))) in RECIPES.iter().zip(want) {
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
        let cucumber = RECIPES[2].amount;
        assert_eq!(cut_amount(cucumber, 0), 50);
        assert_eq!(cut_amount(cucumber, 149), 199);
        assert_eq!(cut_amount(cucumber, 99_999), 199);
        let tuna = RECIPES[4].amount;
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
            Some(&RECIPES[2]),
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
}
