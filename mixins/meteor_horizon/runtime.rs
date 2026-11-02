// Shared by the standalone Cargo library and both injected renderer modules.
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub xray: bool,
    pub fullbright: bool,
}

pub fn flag(value: Option<&str>) -> bool {
    value.is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| Config {
        xray: flag(std::env::var("HORIZON_XRAY").ok().as_deref()),
        fullbright: flag(std::env::var("HORIZON_FULLBRIGHT").ok().as_deref()),
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
