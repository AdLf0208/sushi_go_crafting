// Runs at the head of `Game::native_block_entity` (API 15): the cutting board
// gets a block entity for its one slot and its stroke count.
fn observe_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::sushigocrafting_state::on_block_entity(block, entity);
}
