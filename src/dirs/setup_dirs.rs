use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_data_dir(addon_path: Option<&str>) -> PathBuf {
    let base_dir = ProjectDirs::from("org", "darrienglasser.com", "typeracer").unwrap();
    let safe_dir = base_dir.data_dir();
    let full_dir = &append_if_present(safe_dir, addon_path);

    fs::create_dir_all(&full_dir).expect("Failed to create data dir");
    full_dir.to_path_buf()
}

pub fn get_quote_dir() -> PathBuf {
    create_data_dir(None).join("lang-packs")
}

pub fn get_db_dir() -> PathBuf {
    create_data_dir(Some("stats-db"))
}

fn append_if_present(path: &Path, addon_path: Option<&str>) -> PathBuf {
    match addon_path {
        Some(addon) => path.join(addon),
        None => path.to_path_buf(),
    }
}
