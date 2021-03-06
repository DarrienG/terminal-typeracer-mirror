use config::{Combo, Display, TyperacerConfig};

use crate::config;

const DEFAULT_LANG_PACK_VERSION: &str = "1.0.0";

const DEFAULT_HISTORY_SIZE: usize = 50;

const DEFAULT_COMBO_TRIGGER: usize = 60;

impl Default for TyperacerConfig {
    fn default() -> Self {
        TyperacerConfig {
            lang_packs: None,
            display_settings: Display {
                always_full: false,
                simple_borders: false,
            },
            repo: "https://gitlab.com/ttyperacer/lang-packs.git".to_string(),
            repo_version: DEFAULT_LANG_PACK_VERSION.to_string(),
            extra_repos: vec![],
            history_size: DEFAULT_HISTORY_SIZE,
            combo_config: Combo {
                combo_trigger: DEFAULT_COMBO_TRIGGER,
            },
        }
    }
}
