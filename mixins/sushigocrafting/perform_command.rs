// Runs at the head of `Game::native_perform_command` (API 8): the readout.
fn perform_command(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::sushigocrafting_state::on_perform_command(game, player, command, ran);
}
