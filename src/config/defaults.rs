use crate::config::TyperacerConfig;

const DEFAULT_LANG_PACK_VERSION: &str = "lang-0.3";

const DEFAULT_HISTORY_SIZE: usize = 20;

pub fn construct_default() -> TyperacerConfig {
    TyperacerConfig {
        lang_packs: None,
        repo: "https://gitlab.com/ttyperacer/lang-packs.git".to_string(),
        repo_version: DEFAULT_LANG_PACK_VERSION.to_string(),
        history_size: DEFAULT_HISTORY_SIZE,
    }
}