use super::{Game, Screen};
use horizon_client::mesher::meteor_horizon as runtime;

pub(super) fn active(game: &Game<'_>) -> bool {
    runtime::menu_open() && game.screen == Screen::Options
}
pub(super) fn close(game: &mut Game<'_>) {
    runtime::set_menu_open(false);
    game.screen = Screen::Playing;
    game.keys.clear();
    game.capture(true);
}
pub(super) fn activate(game: &mut Game<'_>, row: u8) {
    if row == 2 {
        close(game);
        return;
    }
    runtime::state().toggle(row);
    if row == 0 {
        game.all_sections_changed();
    }
}
