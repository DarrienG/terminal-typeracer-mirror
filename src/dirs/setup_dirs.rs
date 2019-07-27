use directories::ProjectDirs;
use std::fs;

pub fn create_data_dir() -> String {
    let dirs = ProjectDirs::from("org", "darrienglasser.com", "typeracer").unwrap();
    fs::create_dir_all(dirs.data_dir()).expect("Failed to create data dir");
    dirs.data_dir().to_str().unwrap().to_string()
}

pub fn get_quote_dir() -> String {
    format!("{}/{}", create_data_dir(), "lang-packs").to_string()
}
