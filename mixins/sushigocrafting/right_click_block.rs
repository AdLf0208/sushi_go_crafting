// Runs at the head of `Game::native_right_click_block` (API 21): insert an
// ingredient, stroke with a knife, cut on the sixth stroke, or hand the stack
// back while sneaking.
fn observe_right_click(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::sushigocrafting_state::on_right_click(game, pos, face, canceled);
}
