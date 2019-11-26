use info_render::render;
use itertools::izip;
use std::io::stdin;
use std::{thread, time};
use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::style::Color;
use tui::style::Style;
use tui::terminal::Terminal;
use tui::widgets::Text;

mod info_render;

pub struct InfoData<'a> {
    pub top_text: &'a Vec<Text<'a>>,
    pub bottom_text: &'a Vec<Text<'a>>,
}

static TYPERACER_MAGIC: [&str; 10] = ["t", "t", "y", "p", "e", "r", "a", "c", "e", "r"];
static TYPING_DELAY: [u64; 10] = [144, 80, 144, 144, 144, 100, 105, 95, 80, 100];

pub fn show_info<B: Backend>(terminal: &mut Terminal<B>, typeracer_version: &str) {
    let mut top_text: Vec<Text> = vec![];
    for (type_text, delay) in izip!(TYPERACER_MAGIC.iter(), TYPING_DELAY.iter()) {
        top_text.push(Text::styled(
            type_text.to_string(),
            Style::default().fg(Color::Green),
        ));
        render(
            terminal,
            &InfoData {
                top_text: &top_text,
                bottom_text: &vec![],
            },
        );
        thread::sleep(time::Duration::from_millis(*delay));
    }

    top_text.push(Text::raw(format!(" - version {}\n", typeracer_version)));
    top_text.push(Text::raw("A terminal typeracing game\n"));
    top_text.push(Text::raw(
        "Type through passages to see what the fastest times are you can get!\n\n",
    ));
    top_text.push(Text::raw(
        "repo: https://gitlab.com/ttyperacer/terminal-typeracer\n",
    ));
    top_text.push(Text::raw(
        "main lang packs: https://gitlab.com/ttyperacer/lang-packs",
    ));
    let info_data = InfoData {
        top_text: &top_text,
        bottom_text: &vec![
            Text::styled(
                "\n\nOriginal author: Darrien Glasser\nInspired by Vrinda\n\n",
                Style::default().fg(Color::Gray),
            ),
            Text::raw("^C to return"),
        ],
    };

    render(terminal, &info_data);
    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            if c.unwrap() == Key::Ctrl('c') {
                return;
            }
        }
    }
}
