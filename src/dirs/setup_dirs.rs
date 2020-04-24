use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

/// Create data dir for quotes, database, etc. if they don't exist and return it
pub fn create_data_dir(addon_path: Option<&str>) -> PathBuf {
    let base_dir = ProjectDirs::from("org", "darrienglasser.com", "typeracer").unwrap();
    let safe_dir = base_dir.data_dir();
    let full_dir = &append_if_present(safe_dir, addon_path);

    fs::create_dir_all(&full_dir).expect("Failed to create data dir");
    full_dir.to_path_buf()
}

#[cfg(not(test))]
/// Get dir quotes are in
pub fn get_quote_dir() -> PathBuf {
    create_data_dir(None).join("lang-packs")
}

/// Get path to folder database is in
pub fn get_db_dir() -> PathBuf {
    create_data_dir(Some("stats-db"))
}

/// Get path to where the raw sqlite database file is
pub fn get_db_path() -> PathBuf {
    get_db_dir().join("stats.db")
}

#[cfg(test)]
/// We don't want to actually make any files during tests, so let's just mock out
/// making the path and return a canned one for tests.
pub fn get_quote_dir() -> PathBuf {
    PathBuf::new().join("/home/darrien/.local/share/typeracer/lang-packs")
}

/// Append path to Path if present
fn append_if_present(path: &Path, addon_path: Option<&str>) -> PathBuf {
    match addon_path {
        Some(addon) => path.join(addon),
        None => path.to_path_buf(),
    }
}
