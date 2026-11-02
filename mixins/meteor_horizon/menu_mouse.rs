fn menu_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> std::ops::ControlFlow<Result<()>> {
    if !crate::game::meteor_menu::active(game) {
        return std::ops::ControlFlow::Continue(());
    }
    if pressed && !right {
        let (w, h) = game.gui_size();
        if let Some(row) = horizon_client::mesher::meteor_horizon::menu_hit(w, h, game.cursor) {
            crate::game::meteor_menu::activate(game, row);
        }
    }
    std::ops::ControlFlow::Break(Ok(()))
}
