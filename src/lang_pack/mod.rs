use flate2::read::GzDecoder;
use std::fs::{read_dir, File};
use std::io;
use std::io::{stdin, stdout, Error};
use tar::Archive;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::dirs::setup_dirs;

mod lang_pack_render;

fn download(url: &str, file_path: &str) {
    let mut resp = reqwest::get(url).expect("request failed");
    let mut out = File::create(file_path).expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("Failed to write quote pack to disk");
}

fn expand_lang_pack(file_path: &str, extract_path: &str) -> Result<(), Error> {
    let tar_gz = File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(extract_path)
}

pub fn check_lang_pack() -> bool {
    let data_dir = setup_dirs::create_data_dir();
    read_dir(data_dir).unwrap().count() > 0
}

pub fn retrieve_lang_pack() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let lang_pack_url = "https://gitlab.com/DarrienG/terminal-typeracer/raw/c7a1c5259b21b8faea39312cd013425b3bf8440e/assets/quote-pack.tar.gz";

    let mut step_instruction = "Lang pack (~40Ki) not installed. Would you like to install now? (requires an internet connection)\nYes: y, No: n\n".to_string();
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
            download(lang_pack_url, &file_path);
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
