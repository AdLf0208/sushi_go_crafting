// Runs at the head of `Game::native_block_entity_container` (API 16): one input
// slot, every face, no menu (upstream has no GUI).
fn observe_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::sushigocrafting_state::on_container(block, container);
}
