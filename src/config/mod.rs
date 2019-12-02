use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{stdin, stdout, Error, ErrorKind, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct TyperacerConfig {
    pub lang_packs: Option<LangPacks>,
    pub display_settings: Display,
    pub repo: String,
    pub repo_version: String,
    pub history_size: usize,
    pub combo_config: Combo,
}

#[derive(Debug, Deserialize)]
pub struct RawTyperacerConfig {
    pub lang_packs: Option<LangPacks>,
    pub display_settings: Option<RawDisplay>,
    pub repo: Option<String>,
    pub repo_version: Option<String>,
    pub history_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct LangPacks {
    pub whitelisted: Option<Vec<String>>,
    pub blacklisted: Option<Vec<String>>,
}

// Raw version not yet available. Still deciding how to properly surface
// this to the user.
// Should it go in its own struct? Be part of another? Do we even want to
// surface?
#[derive(Debug, Deserialize)]
pub struct Combo {
    pub combo_trigger: usize,
}

#[derive(Debug, Deserialize)]
pub struct Display {
    pub always_full: bool,
}

#[derive(Debug, Deserialize)]
pub struct RawDisplay {
    pub always_full: Option<bool>,
}

pub mod defaults;
mod validator;

pub fn get_config() -> TyperacerConfig {
    match get_config_raw() {
        Err(parse_err) => {
            println!(
                "Error parsing config file at: {:?}\n\n{:#?}\n",
                get_config_file(),
                parse_err
            );
            println!("Press <ENTER> to continue");
            user_enter();
            defaults::construct_default()
        }
        Ok(v) => match validator::validate_config(v) {
            Err(validate_err) => {
                println!("Error validating config file:\n{:#?}", validate_err);
                println!("Press <ENTER> to continue");
                user_enter();
                defaults::construct_default()
            }
            Ok(cfg) => construct_config(cfg),
        },
    }
}

#[cfg(not(test))]
fn get_config_raw() -> Result<RawTyperacerConfig, Error> {
    let config_buf = get_config_file();
    let mut file_contents = "".to_owned();
    File::open(config_buf)
        .expect("Unable to open config file")
        .read_to_string(&mut file_contents)
        .expect("Unable to read config file");
    let tr_config: Result<RawTyperacerConfig, toml::de::Error> = toml::from_str(&file_contents);
    match tr_config {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::new(ErrorKind::InvalidInput, format!("{}", e))),
    }
}

fn construct_config(raw_config: RawTyperacerConfig) -> TyperacerConfig {
    let default_config = defaults::construct_default();
    TyperacerConfig {
        lang_packs: raw_config.lang_packs,
        display_settings: construct_display(
            raw_config.display_settings,
            default_config.display_settings,
        ),
        combo_config: default_config.combo_config,
        repo: raw_config.repo.unwrap_or(default_config.repo),
        repo_version: raw_config
            .repo_version
            .unwrap_or(default_config.repo_version),
        history_size: raw_config
            .history_size
            .unwrap_or(default_config.history_size),
    }
}

fn construct_display(display_config: Option<RawDisplay>, default_display: Display) -> Display {
    match display_config {
        None => default_display,
        Some(d) => Display {
            always_full: d.always_full.unwrap_or(default_display.always_full),
        },
    }
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

fn user_enter() {
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Unable to read line :(");
}

#[cfg(test)]
fn get_config_raw() -> Result<RawTyperacerConfig, Error> {
    let tr_config: Result<RawTyperacerConfig, toml::de::Error> = toml::from_str("");
    match tr_config {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::new(ErrorKind::InvalidInput, format!("{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests that we provide a sane default for the user.
    /// We should always provide a reliable, sane, default for the user.
    /// Add test to ensure we do.
    fn test_sane_defaults() {
        // Note that get_config_raw is overloaded for tests to give an
        // empty config file
        let config = get_config();
        assert!(config.lang_packs.is_none());
        assert!(config.repo == "https://gitlab.com/ttyperacer/lang-packs.git");
        assert!(config.repo_version == "lang-0.3");
        assert!(config.history_size == 20);
    }
}
