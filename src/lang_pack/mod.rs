use git2::{build, Repository};
use std::fs::{read_dir, File};
use std::io::{stdin, stdout, BufRead, BufReader, Error};
use std::path::Path;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::dirs::setup_dirs;

mod lang_pack_render;

fn download_and_checkout(url: &str, repo_path: &str, data_pack_version: &str) {
    let repo = if Path::new(repo_path).exists() {
        match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("Unable to open repo: {}", e),
        }
    } else {
        match Repository::clone(url, repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("Unable to clone repo: {}", e),
        }
    };

    let mut remote = repo
        .find_remote("origin")
        .expect("Unable to find remote for repo");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.download_tags(git2::AutotagOption::All);
    remote
        .fetch(&[data_pack_version], Some(&mut fetch_options), None)
        .expect("Failed to fetch");

    repo.set_head(&format!("refs/remotes/origin/{}", data_pack_version))
        .expect("Unable to checkout version of lang pack");

    repo.checkout_head(Some(
        build::CheckoutBuilder::new().remove_untracked(true).force(),
    ))
    .expect("Failed to checkout HEAD");
}

fn check_proper_version(lang_pack_version: &str, data_dir: &str) -> bool {
    // Somehow the file doesn't exist, so we should just get the right version
    let version_file = format!("{}/version", data_dir);
    if !Path::new(&version_file).exists() {
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

pub fn check_lang_pack(lang_pack_version: &str) -> bool {
    let quote_dir = setup_dirs::get_quote_dir();
    if Path::new(&quote_dir).exists() && read_dir(&quote_dir).unwrap().count() > 0 {
        check_proper_version(lang_pack_version, &quote_dir)
    } else {
        false
    }
}

/// Retrieves the langpack with the given version.
/// Returns true if the user wants to continue, false otherwise
pub fn retrieve_lang_pack(data_pack_version: &str) -> Result<bool, Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let lang_pack_url = "https://gitlab.com/ttyperacer/lang-packs.git";

    let mut step_instruction = "Lang pack (1.5Mi installed) not on version compatible with your typeracer, install the proper version? (requires an internet connection)\nYes: y, No: n\n".to_string();
    let mut step_count = 0;

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
                                "\nMaking data dir at: {}\n",
                                setup_dirs::create_data_dir()
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
                step_instruction.push_str("Downloading and setting up lang packs...\n");
                download_and_checkout(
                    lang_pack_url,
                    &setup_dirs::get_quote_dir(),
                    data_pack_version,
                );
                step_instruction.push_str(
                        "Lang pack downloaded and ready to go!\nPress any key to continue or ^C to exit.\n",
                    )
            }
            _ => {
                let c = stdin.keys().next().unwrap();
                return result.map(|()| match c.unwrap() {
                    Key::Ctrl('c') => false,
                    _ => true,
                });
            }
        }
    }
}
