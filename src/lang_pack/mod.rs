use git2::{
    Error as GitError, {build, Repository},
};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::{
    collections::{HashMap, HashSet},
    fs::{read_dir, remove_dir_all, File},
    io::{stdin, stdout, BufRead, BufReader, Error},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use crate::{config::TyperacerConfig, dirs::setup_dirs};

mod lang_pack_render;

fn download_and_checkout(
    url: &str,
    repo_path: &Path,
    data_pack_version: &str,
) -> Result<(), GitError> {
    let repo = if Path::new(repo_path).exists() {
        Repository::open(repo_path)?
    } else {
        Repository::clone(url, repo_path)?
    };

    let mut remote = repo
        .find_remote("origin")
        .expect("Unable to find remote for repo");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.download_tags(git2::AutotagOption::All);
    remote.fetch(&[data_pack_version], Some(&mut fetch_options), None)?;

    repo.set_head(&format!("refs/remotes/origin/{}", data_pack_version))?;

    repo.checkout_head(Some(
        build::CheckoutBuilder::new().remove_untracked(true).force(),
    ))?;

    Ok(())
}

fn check_proper_version(lang_pack_version: &str, data_dir: &Path) -> bool {
    // Somehow the file doesn't exist, so we should just get the right version
    let version_file = data_dir.join("version");
    if !version_file.exists() {
        return false;
    }

    let version_file = File::open(&version_file).expect("Failed to read version file");
    let mut version_text: Vec<String> = vec![];
    for line in BufReader::new(version_file).lines() {
        version_text.push(line.unwrap());
    }
    if version_text.len() < 2 {
        // Corrupt version file. We will assume something is wrong and we
        // should get a new version
        false
    } else {
        version_text[0].trim() == lang_pack_version
    }
}

pub fn check_lang_pack(config: &TyperacerConfig) -> bool {
    check_main_lang_pack(&config.repo_version) && check_extra_lang_packs(config)
}

fn check_main_lang_pack(lang_pack_version: &str) -> bool {
    let quote_dir = setup_dirs::get_quote_dirs().main_pack_dir;
    if quote_dir.exists() && read_dir(&quote_dir).unwrap().count() > 0 {
        check_proper_version(lang_pack_version, &quote_dir)
    } else {
        false
    }
}

/// Get extra directories as map of {simple simple name, actual path on computer}
fn extra_repos_as_map(extra_dir_location: &Path) -> HashMap<String, PathBuf> {
    extra_dir_location
        .read_dir()
        .expect("Directory disappeared when we tried to read it.")
        .map(|wrapped_dir| wrapped_dir.expect("Unable to read dir"))
        .map(|dir| (dir.file_name().to_string_lossy().to_string(), dir.path()))
        .collect::<HashMap<String, PathBuf>>()
}

fn check_extra_lang_packs(config: &TyperacerConfig) -> bool {
    let mut clean = true;
    let extra_dir_location = setup_dirs::get_quote_dirs().extra_pack_dir;
    if !extra_dir_location.exists() {
        return false;
    }

    let extra_repos = extra_repos_as_map(&extra_dir_location);
    clean = clean && config.extra_repos.len() == extra_repos.len();

    for repo in config.extra_repos.iter() {
        if !extra_repos.contains_key(&repo.name) {
            clean = false;
        } else {
            clean = clean && check_proper_version(&repo.version, &extra_repos[&repo.name]);
        }
    }

    clean
}

fn repos_to_clean(config: &TyperacerConfig) -> HashSet<PathBuf> {
    let extra_dir_location = setup_dirs::get_quote_dirs().extra_pack_dir;
    if !extra_dir_location.exists() {
        return Default::default();
    }
    let mut currently_installed_repos = extra_repos_as_map(&extra_dir_location);

    for repo in config.extra_repos.iter() {
        currently_installed_repos.remove(&repo.name);
    }

    currently_installed_repos.values().cloned().collect()
}

fn clean_extra_repos(stale_repos: &HashSet<PathBuf>) -> bool {
    let mut success = true;
    for stale_repo in stale_repos.iter() {
        success = success && remove_dir_all(stale_repo).is_ok();
    }

    success
}

/// Retrieves the langpack with the given version.
/// Returns true if the user wants to continue, false otherwise
pub fn retrieve_lang_pack(
    data_pack_version: &str,
    typeracer_config: &TyperacerConfig,
) -> Result<bool, Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let mut step_instruction =
        "Found updated lang packs! Update to the latest versions? (y/n)\n".to_string();
    let mut step_count = 0;
    let mut stale_repo_paths: HashSet<PathBuf> = Default::default();

    let result: Result<(), Error> = Ok(());

    loop {
        let stdin = stdin();
        lang_pack_render::render(&mut terminal, &step_instruction);
        match step_count {
            0 => {
                for c in stdin.keys() {
                    match c.unwrap() {
                        Key::Char('y') | Key::Char('Y') => {
                            step_count += 1;
                            step_instruction.push_str(&format!(
                                "Making data dir at: {}\n",
                                setup_dirs::create_data_dir(None).to_str().unwrap()
                            ));
                            break;
                        }
                        Key::Char('n') | Key::Char('N') => return Ok(false),
                        _ => (),
                    }
                }
            }
            1 => {
                step_count += 1;
                step_instruction.push_str("Downloading and setting up lang packs (this may take a while with slow internet)...\n");
            }
            2 => {
                step_count += 1;
                match download_and_checkout(
                    &typeracer_config.repo,
                    &setup_dirs::get_quote_dirs().main_pack_dir,
                    data_pack_version,
                ) {
                    Ok(()) => {
                        step_instruction.push_str(&format!(
                            "Main lang pack downloaded and ready to go!\n{}",
                            get_extra_repos_string(typeracer_config)
                        ));
                    }
                    Err(e) => {
                        step_count = 5;
                        step_instruction.push_str(&format!(
                            "Trouble downloading main repo at: {} error: {} please try again\n",
                            typeracer_config.repo, e
                        ));
                    }
                };
            }
            3 => {
                step_count += 1;
                for repo in typeracer_config.extra_repos.iter() {
                    match download_and_checkout(
                        &repo.url,
                        &setup_dirs::get_quote_dirs().extra_pack_dir.join(&repo.name),
                        &repo.version,
                    ) {
                        Ok(()) => {
                            step_instruction.push_str(&format!(
                                "Downloaded and installed user repo: {} as {}\n",
                                repo.url, repo.name
                            ));
                        }
                        Err(e) => step_instruction.push_str(&format!(
                            "Trouble downloading extra repo: {}, error: {}. Please try again.\n",
                            repo.url, e
                        )),
                    };
                }
                stale_repo_paths = repos_to_clean(typeracer_config);
                if !stale_repo_paths.is_empty() {
                    step_instruction.push_str(&format!(
                        "Found unreferenced repos:\n{}\nWould you like to delete them? (y/n)\n",
                        stale_repo_paths
                            .iter()
                            .map(|path| format!("- {}", path.to_str().unwrap()))
                            .join("\n")
                    ));
                } else {
                    step_count += 1;
                }
            }
            4 => {
                step_count += 1;
                for c in stdin.keys() {
                    match c.unwrap() {
                        Key::Char('y') | Key::Char('Y') => {
                            step_instruction.push_str(&get_cleaned_repos_string(
                                clean_extra_repos(&stale_repo_paths),
                            ));
                            break;
                        }
                        Key::Char('n') | Key::Char('N') => {
                            step_instruction.push_str("Requested not to delete old repos.\n");
                            break;
                        }
                        _ => (),
                    }
                }
            }
            5 => {
                step_count += 1;
                step_instruction.push_str("Press any key to continue or ^C to exit.\n");
            }
            _ => {
                let c = stdin.keys().find_map(Result::ok);
                return result.map(|()| match c.unwrap() {
                    Key::Ctrl('c') => false,
                    _ => true,
                });
            }
        }
    }
}

fn get_extra_repos_string(config: &TyperacerConfig) -> String {
    if config.extra_repos.is_empty() {
        "".to_owned()
    } else {
        format!(
            "Downloading [{}] extra user configured repos...\n",
            config.extra_repos.len()
        )
    }
}

fn get_cleaned_repos_string(successful: bool) -> String {
    if successful {
        "Successfully deleted old repos!\n".to_owned()
    } else {
        "Issue deleting old repos. Try again later.\n".to_owned()
    }
}
