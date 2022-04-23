use crossbeam_channel::Receiver;
use git_version::git_version;
use info_render::render;
use itertools::izip;
use std::time::Duration;
use std::{thread, time};
use termion::event::Key;
use tui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Terminal,
    text::Text,
};

mod info_render;

pub struct InfoData<'a> {
    pub top_text: &'a Text<'a>,
    pub bottom_text: &'a Text<'a>,
}

const MAGIC_AMT: usize = 10;

static TYPERACER_MAGIC: [&str; MAGIC_AMT] = ["t", "t", "y", "p", "e", "r", "a", "c", "e", "r"];
static TYPING_DELAY: [u64; MAGIC_AMT] = [144, 80, 144, 144, 144, 100, 105, 95, 80, 100];

pub fn show_info<B: Backend>(
    terminal: &mut Terminal<B>,
    input_receiver: &Receiver<Key>,
    typeracer_version: &str,
) {
    let version_string = &mut format!(
        " - version {} - BUILD {}\n",
        typeracer_version,
        git_version!()
    );

    let mut magic = TYPERACER_MAGIC.to_vec();
    magic.push(version_string);

    let mut delay = TYPING_DELAY.to_vec();
    delay.push(0);

    let mut ttyperacer = "".to_owned();
    let mut top_text = Text::default();
    for (type_text, delay) in izip!(magic.iter(), delay.iter()) {
        ttyperacer.push_str(type_text);
        top_text = Text::styled(
            (&ttyperacer).to_string(),
            Style::default().fg(if dirty_commit(&version_string) {
                Color::Red
            } else {
                Color::Green
            }),
        );

        let tmp_text = Text::default();

        render(
            terminal,
            &InfoData {
                top_text: &top_text,
                bottom_text: &tmp_text,
            },
        );
        thread::sleep(time::Duration::from_millis(*delay));
    }

    top_text.extend(Text::raw("A terminal typeracing game\n"));
    top_text.extend(Text::raw(
        "Type through passages to see what the fastest times are you can get!\n\n",
    ));
    top_text.extend(Text::raw(
        "repo: https://gitlab.com/ttyperacer/terminal-typeracer\n",
    ));
    top_text.extend(Text::raw(
        "main lang packs: https://gitlab.com/ttyperacer/lang-packs\n",
    ));
    top_text.extend(Text::raw(format!(
        "docs: https://gitlab.com/ttyperacer/terminal-typeracer/tree/v{}/docs\n\n",
        typeracer_version
    )));
    top_text.extend(Text::raw(format!(
        "current release notes: https://gitlab.com/ttyperacer/terminal-typeracer/-/tags/v{}\n",
        typeracer_version
    )));
    top_text.extend(Text::raw(
        "all releases: https://gitlab.com/ttyperacer/terminal-typeracer/-/releases",
    ));

    let mut bottom_text: Text = Text::styled(
        "\n\nOriginal author: Darrien Glasser\nInspired by Vrinda\n\n",
        Style::default().fg(Color::LightBlue),
    );
    bottom_text.extend(Text::raw("^C to return"));

    let info_data = InfoData {
        top_text: &top_text,
        bottom_text: &bottom_text,
    };

    loop {
        render(terminal, &info_data);
        let recv_result = input_receiver.recv_timeout(Duration::from_millis(500));
        if recv_result.is_err() {
            // just didn't get anything, let's keep going
            continue;
        }

        if recv_result.unwrap() == Key::Ctrl('c') {
            return;
        }
    }
}

// A not "true" release build.
// We find this out by either seeing if it is a debug build (e.g. the git version shows up twice)
// or if we see modified in the version (e.g. built on a dirty git commit).
fn dirty_commit(version_string: &str) -> bool {
    version_string.contains("modified")
        || version_string
            .split(git_version!())
            .collect::<Vec<&str>>()
            .len()
            > 2
}
