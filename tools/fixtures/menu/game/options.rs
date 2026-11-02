use crate::Result;
use super::Game;
use winit::keyboard::KeyCode;
impl Game<'_> {
    pub(super) fn open_options(&mut self, _: bool) { self.open_options_at(false, ()); }
    pub(super) fn open_options_at(&mut self, from_title: bool, page: ()) { self.screen = super::Screen::Options; self.keys.clear(); }
    pub(super) fn gui_size(&self) -> (f32, f32) { (320.0, 240.0) }
    pub(super) fn draw_options(&mut self, batch: &mut crate::hud::HudBatch) {
        batch.labels.push("original options".into());
    }
    pub(super) fn options_mouse(&mut self, pressed: bool, right: bool) -> Result<()> { Ok(()) }
    pub(super) fn options_key(&mut self, key: KeyCode, _text: Option<&str>) -> Result<()> { Ok(()) }
}
