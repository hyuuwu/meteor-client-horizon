fn menu_key(
    game: &mut crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
) -> std::ops::ControlFlow<Result<()>> {
    if !crate::game::meteor_menu::active(game) {
        return std::ops::ControlFlow::Continue(());
    }
    use horizon_client::mesher::meteor_horizon as runtime;
    use winit::keyboard::KeyCode;
    match key {
        KeyCode::Escape => crate::game::meteor_menu::close(game),
        KeyCode::ArrowUp => runtime::state().select(-1),
        KeyCode::ArrowDown | KeyCode::Tab => runtime::state().select(1),
        KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
            crate::game::meteor_menu::activate(game, runtime::state().selection())
        }
        _ => {}
    }
    std::ops::ControlFlow::Break(Ok(()))
}
