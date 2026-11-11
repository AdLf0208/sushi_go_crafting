// Runs before content loads (`native_component_types`, API 65): registers the
// AmountItem's `sushigocrafting:amount` component, so a cut's stack carries it
// through saves and the network and Horizon knows its type.
fn register_component_types(types: &mut Vec<String>) {
    crate::game::sushigocrafting_state::register_component_types(types);
}
