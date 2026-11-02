fn draw_menu(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
) -> std::ops::ControlFlow<()> {
    if !crate::game::meteor_menu::active(game) {
        return std::ops::ControlFlow::Continue(());
    }
    use horizon_client::mesher::meteor_horizon as runtime;
    let (w, h) = (batch.width, batch.height);
    let (x, y) = runtime::menu_origin(w, h);
    let atlas = &game.gui.atlas;
    batch.fill(atlas, 0.0, 0.0, w, h, 0xa0000000);
    batch.fill(atlas, x, y, 220.0, 150.0, 0xf0221820);
    batch.fill(atlas, x, y, 220.0, 2.0, 0xffff70b0);
    batch.text(
        atlas,
        "METEOR / HORIZON",
        x + 12.0,
        y + 13.0,
        u32::MAX,
        true,
    );
    let config = runtime::config();
    let labels = [
        format!("Xray: {}", if config.xray { "ON" } else { "OFF" }),
        format!(
            "Fullbright: {}",
            if config.fullbright { "ON" } else { "OFF" }
        ),
        "Done".into(),
    ];
    let hovered = runtime::menu_hit(w, h, game.cursor);
    for (row, label) in labels.iter().enumerate() {
        let by = y + 35.0 + row as f32 * 28.0;
        let focused = hovered == Some(row as u8) || runtime::state().selection() == row as u8;
        batch.fill(
            atlas,
            x + 12.0,
            by,
            196.0,
            24.0,
            if focused { 0xff704050 } else { 0xff382830 },
        );
        batch.text(atlas, label, x + 20.0, by + 8.0, u32::MAX, true);
    }
    batch.text(
        atlas,
        "Click / arrows + Enter / Esc",
        x + 12.0,
        y + 130.0,
        0xffc0c0c0,
        true,
    );
    std::ops::ControlFlow::Break(())
}
