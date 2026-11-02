// Shared by the standalone Cargo library and the injected renderer module.
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub xray: bool,
    pub fullbright: bool,
}

pub fn flag(value: Option<&str>) -> bool {
    value.is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

pub struct State {
    flags: AtomicU8,
    pub menu_open: AtomicBool,
    selection: AtomicU8,
}
impl State {
    pub fn new(config: Config) -> Self {
        Self {
            flags: AtomicU8::new(u8::from(config.xray) | (u8::from(config.fullbright) << 1)),
            menu_open: AtomicBool::new(false),
            selection: AtomicU8::new(0),
        }
    }
    pub fn config(&self) -> Config {
        let flags = self.flags.load(Ordering::Acquire);
        Config {
            xray: flags & 1 != 0,
            fullbright: flags & 2 != 0,
        }
    }
    pub fn toggle(&self, row: u8) -> Config {
        if row < 2 {
            self.flags.fetch_xor(1 << row, Ordering::AcqRel);
            if row == 1 {
                self.flags.fetch_or(4, Ordering::AcqRel);
            }
        }
        self.config()
    }
    pub fn fullbright_override(&self) -> Option<bool> {
        let flags = self.flags.load(Ordering::Acquire);
        (flags & 6 != 0).then_some(flags & 2 != 0)
    }
    pub fn select(&self, direction: i8) {
        let row = (self.selection() as i8 + direction).rem_euclid(3) as u8;
        self.selection.store(row, Ordering::Relaxed);
    }
    pub fn selection(&self) -> u8 {
        self.selection.load(Ordering::Relaxed)
    }
}

pub fn state() -> &'static State {
    static STATE: OnceLock<State> = OnceLock::new();
    STATE.get_or_init(|| {
        State::new(Config {
            xray: flag(std::env::var("HORIZON_XRAY").ok().as_deref()),
            fullbright: flag(std::env::var("HORIZON_FULLBRIGHT").ok().as_deref()),
        })
    })
}
pub fn config() -> Config {
    state().config()
}
pub fn menu_open() -> bool {
    state().menu_open.load(Ordering::Relaxed)
}
pub fn set_menu_open(open: bool) {
    state().menu_open.store(open, Ordering::Relaxed);
}

pub fn menu_origin(width: f32, height: f32) -> (f32, f32) {
    ((width - 220.0) / 2.0, ((height - 150.0) / 2.0).max(4.0))
}
pub fn menu_hit(width: f32, height: f32, cursor: (f32, f32)) -> Option<u8> {
    let (x, y) = menu_origin(width, height);
    (0..3).find(|&row| {
        cursor.0 >= x + 12.0
            && cursor.0 < x + 208.0
            && cursor.1 >= y + 35.0 + f32::from(row) * 28.0
            && cursor.1 < y + 59.0 + f32::from(row) * 28.0
    })
}

pub fn is_ore(name: &str) -> bool {
    matches!(
        name,
        "minecraft:coal_ore"
            | "minecraft:deepslate_coal_ore"
            | "minecraft:copper_ore"
            | "minecraft:deepslate_copper_ore"
            | "minecraft:iron_ore"
            | "minecraft:deepslate_iron_ore"
            | "minecraft:gold_ore"
            | "minecraft:deepslate_gold_ore"
            | "minecraft:redstone_ore"
            | "minecraft:deepslate_redstone_ore"
            | "minecraft:lapis_ore"
            | "minecraft:deepslate_lapis_ore"
            | "minecraft:diamond_ore"
            | "minecraft:deepslate_diamond_ore"
            | "minecraft:emerald_ore"
            | "minecraft:deepslate_emerald_ore"
            | "minecraft:nether_gold_ore"
            | "minecraft:nether_quartz_ore"
            | "minecraft:ancient_debris"
    )
}

pub fn filter_blocks(blocks: &mut [u32], mut selected: impl FnMut(u32) -> bool) {
    for block in blocks {
        if *block != 0 && !selected(*block) {
            *block = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_toggles_are_independent_and_reversible() {
        let state = State::new(Config::default());
        assert!(state.toggle(0).xray);
        assert!(!state.config().fullbright);
        assert!(state.toggle(1).fullbright);
        assert!(!state.toggle(0).xray);
        assert!(!state.toggle(1).fullbright);
        assert_eq!(state.fullbright_override(), Some(false));
        state.select(-1);
        assert_eq!(state.selection(), 2);
        state.select(1);
        assert_eq!(state.selection(), 0);
    }
    #[test]
    fn menu_hit_testing_matches_scaled_layout() {
        for (w, h) in [(320.0, 240.0), (960.0, 540.0)] {
            let (x, y) = menu_origin(w, h);
            for row in 0..3 {
                assert_eq!(
                    menu_hit(w, h, (x + 20.0, y + 40.0 + f32::from(row) * 28.0)),
                    Some(row)
                );
            }
            assert_eq!(menu_hit(w, h, (x - 1.0, y + 40.0)), None);
            assert_eq!(menu_hit(w, h, (x + 20.0, y + 60.0)), None);
        }
    }

    #[test]
    fn switches_are_explicit() {
        for value in [None, Some("0"), Some("false"), Some("yes"), Some("")] {
            assert!(!flag(value));
        }
        for value in [Some("1"), Some("true"), Some("TRUE")] {
            assert!(flag(value));
        }
        assert_eq!(
            Config::default(),
            Config {
                xray: false,
                fullbright: false
            }
        );
    }

    #[test]
    fn only_vanilla_ores_are_selected() {
        for name in [
            "minecraft:diamond_ore",
            "minecraft:deepslate_redstone_ore",
            "minecraft:ancient_debris",
        ] {
            assert!(is_ore(name));
        }
        for name in [
            "minecraft:stone",
            "minecraft:lava",
            "minecraft:diamond_block",
            "other:diamond_ore",
        ] {
            assert!(!is_ore(name));
        }
    }

    #[test]
    fn filtering_preserves_selected_states_and_input_world() {
        let world = [0, 1, 2, 3, 2];
        let mut mesh = world;
        filter_blocks(&mut mesh, |id| id == 2);
        assert_eq!(mesh, [0, 0, 2, 0, 2]);
        assert_eq!(world, [0, 1, 2, 3, 2]);
    }
}
