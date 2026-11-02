use crate::Result;
use crate::hud::{HudAtlas, HudBatch};
use std::collections::HashSet;
use winit::keyboard::KeyCode;
pub mod options;
#[derive(PartialEq, Eq)]
pub enum Screen { Playing, Options, Inventory }
pub struct Extras { pub fullbright: bool }
pub struct Settings { pub extras: Extras }
pub struct Gui { pub atlas: HudAtlas }
pub struct Packs { pub prompt: Option<()> }
pub struct Game<'a> {
    pub marker: &'a (), pub screen: Screen, pub keys: HashSet<KeyCode>,
    pub settings: Settings, pub gui: Gui, pub cursor: (f32, f32),
    pub player: Option<()>, pub title: Option<()>, pub sleeping: Option<()>, pub server_packs: Packs,
    pub captured: bool, pub remesh: usize, pub applied: usize, pub original_keys: usize,
}
impl Game<'_> {
    fn key_pressed(&mut self, key: KeyCode) -> Result<()> {
        self.original_keys += 1; Ok(())
    }
    fn capture(&mut self, capture: bool) { self.captured = capture; }
    fn dialog_open(&self) -> bool { false }
    fn all_sections_changed(&mut self) { self.remesh += 1; }
    fn apply_settings(&mut self) { self.applied += 1; }
}
#[test]
fn menu_opens_draws_toggles_and_closes_without_gameplay_input() {
    use crate::mesher::meteor_horizon as runtime;
    let mut game = Game { marker: &(), screen: Screen::Playing, keys: HashSet::new(),
        settings: Settings { extras: Extras { fullbright: false } }, gui: Gui { atlas: HudAtlas },
        cursor: (0.0, 0.0), player: Some(()), title: None, sleeping: None, server_packs: Packs { prompt: None },
        captured: true, remesh: 0, applied: 0, original_keys: 0 };
    game.key_pressed(KeyCode::ShiftRight).unwrap();
    assert!(runtime::menu_open()); assert!(!game.captured); assert_eq!(game.original_keys, 0);
    let mut batch = HudBatch { width: 320.0, height: 240.0, labels: Vec::new() };
    game.draw_options(&mut batch);
    assert!(batch.labels.iter().any(|label| label.contains("METEOR / HORIZON")));
    let before = runtime::config();
    let (x, y) = runtime::menu_origin(320.0, 240.0);
    game.cursor = (x + 20.0, y + 40.0);
    game.options_mouse(true, false).unwrap();
    assert_eq!(runtime::config().xray, !before.xray); assert_eq!(game.remesh, 1);
    game.options_mouse(false, false).unwrap(); assert_eq!(game.remesh, 1);
    game.options_key(KeyCode::ArrowDown, None).unwrap();
    game.options_key(KeyCode::Enter, None).unwrap();
    assert_eq!(runtime::config().fullbright, !before.fullbright);
    assert!(!game.settings.extras.fullbright); assert_eq!(game.applied, 0);
    game.options_key(KeyCode::Escape, None).unwrap();
    assert!(!runtime::menu_open()); assert!(game.captured); assert!(game.screen == Screen::Playing);
    game.screen = Screen::Inventory;
    game.key_pressed(KeyCode::ShiftRight).unwrap(); assert!(!runtime::menu_open());
    assert_eq!(game.original_keys, 1);
    runtime::set_menu_open(true);
    game.open_options(false);
    assert!(!runtime::menu_open());
    batch.labels.clear(); game.draw_options(&mut batch);
    assert_eq!(batch.labels, vec!["original options"]);
    // A session override must survive reopening even when the native option is on.
    if runtime::config().fullbright { runtime::state().toggle(1); }
    game.settings.extras.fullbright = true;
    game.screen = Screen::Playing;
    game.key_pressed(KeyCode::ShiftRight).unwrap();
    assert!(!runtime::config().fullbright);
    game.options_key(KeyCode::Escape, None).unwrap();
}
