use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct TyperacerConfig {
    pub lang_packs: Option<LangPacks>,
    pub repo: Option<String>,
    pub repo_version: Option<String>,
    pub history_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct LangPacks {
    pub whitelisted: Option<Vec<String>>,
    pub blacklisted: Option<Vec<String>>,
}

mod validator;

pub fn get_config() -> Result<TyperacerConfig, Error> {
    validator::validate_config(get_config_raw())
}

fn get_config_raw() -> TyperacerConfig {
    let config_buf = get_config_file();
    let mut file_contents = "".to_owned();
    File::open(config_buf)
        .expect("Unable to open config file")
        .read_to_string(&mut file_contents)
        .expect("Unable to read config file");
    toml::from_str(&file_contents).expect("Unable to parse config file to valid toml")
}

fn get_config_file() -> PathBuf {
    let mut config_dir = create_config_dir();
    config_dir.push("config.toml");
    if config_dir.exists() {
        config_dir
    } else {
        File::create(&config_dir).unwrap();
        config_dir
    }
}

fn create_config_dir() -> PathBuf {
    let dirs = ProjectDirs::from("org", "darrienglasser.com", "typeracer").unwrap();
    fs::create_dir_all(dirs.config_dir()).expect("Failed to create config dir");
    PathBuf::from(dirs.config_dir())
}
