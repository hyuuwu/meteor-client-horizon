fn open_menu(
    game: &mut crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
) -> std::ops::ControlFlow<Result<()>> {
    if key != winit::keyboard::KeyCode::ShiftRight
        || game.screen != crate::game::Screen::Playing
        || game.player.is_none()
        || game.title.is_some()
        || game.sleeping.is_some()
        || game.server_packs.prompt.is_some()
        || game.dialog_open()
    {
        return std::ops::ControlFlow::Continue(());
    }
    use horizon_client::mesher::meteor_horizon as runtime;
    // Reflect an already enabled native Fullbright setting when the menu first opens.
    if game.settings.extras.fullbright && runtime::state().fullbright_override().is_none() {
        runtime::state().toggle(1);
    }
    game.open_options(false);
    runtime::set_menu_open(true);
    game.capture(false);
    std::ops::ControlFlow::Break(Ok(()))
}
