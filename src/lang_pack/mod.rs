use flate2::read::GzDecoder;
use git2::Repository;
use std::fs;
use std::fs::{read_dir, File};
use std::io::{stdin, stdout, BufRead, BufReader, Error};
use std::path::Path;
use tar::Archive;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::dirs::setup_dirs;

mod lang_pack_render;

fn download_and_checkout(
    url: &str,
    data_dir: &str,
    quote_pack_dest: &str,
    data_pack_version: &str,
) {
    let repo_path = &format!("{}/terminal-typeracer", data_dir);
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

    repo.set_head(&format!("refs/heads/{}", data_pack_version))
        .expect("Unable to checkout version of lang pack");

    // The case where we can't remove the old one is usually that the
    // old one doesn't exist, so we don't need to worry about checking
    // for err here.
    let _ = fs::remove_file(quote_pack_dest);
    fs::copy(
        format!("{}/default/quote-pack.tar.gz", repo_path),
        quote_pack_dest,
    )
    .expect("Unable to copy lang pack to destination dir");
}

fn expand_lang_pack(file_path: &str, extract_path: &str) -> Result<(), Error> {
    let tar_gz = File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(extract_path)
}

fn check_proper_version(lang_pack_version: &str, data_dir: &str) -> bool {
    let version_file =
        File::open(format!("{}/version", data_dir)).expect("Failed to read version file");
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
    let data_dir = setup_dirs::create_data_dir();
    let quote_dir = &format!("{}/quote-pack", data_dir);
    if Path::new(quote_dir).exists() && read_dir(quote_dir).unwrap().count() > 0 {
        check_proper_version(lang_pack_version, &setup_dirs::get_quote_dir())
    } else {
        false
    }
}

pub fn retrieve_lang_pack(data_pack_version: &str) -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let lang_pack_url = "https://gitlab.com/ttyperacer/lang-packs.git";

    let mut step_instruction = "Lang pack (100Ki) not on version compatible with your typeracer, install the proper version? (requires an internet connection)\nYes: y, No: n\n".to_string();
    let mut step_count = 0;

    let mut data_dir: String = "".to_string();
    let mut file_path: String = "".to_string();

    let mut result: Result<(), Error> = Ok(());

    loop {
        let stdin = stdin();
        lang_pack_render::render(&mut terminal, &step_instruction);
        if step_count == 0 {
            for c in stdin.keys() {
                let checked = c.unwrap();
                if checked == Key::Char('y') {
                    step_count += 1;
                    data_dir = setup_dirs::create_data_dir();
                    step_instruction.push_str(&format!("\nMaking data dir at: {}\n", data_dir));
                    break;
                }
                if checked == Key::Char('n') {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "User wants to exit",
                    ));
                }
            }
        } else if step_count == 1 {
            step_count += 1;
            step_instruction.push_str("Downloading lang pack...\n");
            file_path = format!("{}/{}", &data_dir, "quote-pack.tar.gz");
            download_and_checkout(lang_pack_url, &data_dir, &file_path, data_pack_version);
            step_instruction.push_str("Lang pack downloaded!\n");
        } else if step_count == 2 {
            step_count += 1;
            step_instruction.push_str("Extracting lang pack.\n");
            result = expand_lang_pack(&file_path, &data_dir);
            if result.is_err() {
                step_instruction.push_str(
                    "Failed to extract lang pack. Please quit and try again.\n^D to exit.\n",
                );
            } else {
                step_instruction
                    .push_str("Lang pack downloaded and ready to go!\n^D to continue\n");
            }
        } else {
            for c in stdin.keys() {
                if c.unwrap() == Key::Ctrl('d') {
                    return result;
                }
            }
        }
    }
}
