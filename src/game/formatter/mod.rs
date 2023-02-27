use tui::style::Modifier;
use tui::{
    style::{Color, Style},
    text::Span,
};

use crate::game::indexer;
use crate::game::word_processing::GameMode;

#[derive(Debug, Clone)]
pub struct FormattedTexts<'a> {
    pub passage: Vec<Span<'a>>,
    pub input: Vec<Span<'a>>,
    pub error: bool,
    pub complete: bool,
}

/// Should be the final formatting call.
/// Sets formatted texts fields to expected completion settings.
/// Reformats the entire passage from scratch in the case that the user is
/// running with display_settings.always_max=false.
/// If they are, they will only see the final word, but showing the whole
/// passage to them now that it is complete is a much better user experience.
pub fn get_reformatted_complete_texts<'a>(
    game_mode: &GameMode,
    words: &[&str],
) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(game_mode, words, Color::Green, "COMPLETE", false)
}

pub fn get_reformatted_failed_texts<'a>(
    game_mode: &GameMode,
    words: &[&str],
) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(game_mode, words, Color::Red, "FAIL", true)
}

/// Get fully formatted versions of the passage, and the user's input.
pub fn get_formatted_texts<'a>(
    game_mode: &GameMode,
    words: &[&str],
    user_input: &str,
    current_word_idx: usize,
    last_input_char: char,
    new_char: bool,
    mut formatted_passage: Vec<Span<'a>>,
) -> FormattedTexts<'a> {
    let user_has_err = !indexer::check_like_word(words[current_word_idx], user_input);
    let current_word_idx =
        indexer::get_maybe_decremented_idx(game_mode, user_has_err, new_char, current_word_idx);

    let (formatted_passage_word, formatted_input) = get_formatted_words(
        game_mode,
        words[current_word_idx],
        user_input,
        last_input_char,
        new_char,
    );

    let starting_idx = indexer::get_starting_idx(game_mode, words, current_word_idx);

    formatted_passage[starting_idx..(formatted_passage_word.len() + starting_idx)]
        .clone_from_slice(&formatted_passage_word[..]);

    FormattedTexts {
        passage: formatted_passage,
        input: formatted_input,
        error: user_has_err,
        complete: false,
    }
}

/// Get formatted texts with the assumption the word we are typing is the first word.
pub fn get_formatted_texts_line_mode<'a>(
    game_mode: &GameMode,
    current_word: &str,
    user_input: &str,
    last_input_char: char,
    new_char: bool,
    mut formatted_passage: Vec<Span<'a>>,
) -> FormattedTexts<'a> {
    let (formatted_passage_word, formatted_input) = get_formatted_words(
        game_mode,
        current_word,
        user_input,
        last_input_char,
        new_char,
    );
    formatted_passage[0..(formatted_passage_word.len())]
        .clone_from_slice(&formatted_passage_word[..]);

    FormattedTexts {
        passage: formatted_passage,
        input: formatted_input,
        error: !indexer::check_like_word(current_word, user_input),
        complete: false,
    }
}

/// Get formatted version of a single word in a passage and the user's current input.
///
/// All similar characters up until the first different character are highlighted with green.
///
/// On an erroneous character:
/// - The first error character in the passage's word is highlighted with red and the rest unformatted.
/// - The entirety of the user's input is colored red.
///
/// Returns a tuple with the formatted version of the: word and the input.
fn get_formatted_words<'a>(
    game_mode: &GameMode,
    word: &str,
    input: &str,
    last_input_char: char,
    new_char: bool,
) -> (Vec<Span<'a>>, Vec<Span<'a>>) {
    let indexable_word: Vec<char> = word.chars().collect();
    let indexable_input: Vec<char> = input.chars().collect();
    let idx_word_count = indexable_word.len();
    let idx_input_count = indexable_input.len();

    let mut formatted_word: Vec<Span> = Vec::new();
    let mut formatted_input: Vec<Span> = Vec::new();
    let mut word_dex = 0;

    let err = !indexer::check_like_word(word, input);

    // Make all of the user's input white on red
    for raw_input in indexable_input.iter() {
        let style = if err {
            Style::default().bg(Color::Red).fg(Color::White)
        } else {
            Style::default().fg(Color::Green)
        };

        let input = if *raw_input == ' ' {
            style
                .add_modifier(Modifier::ITALIC)
                .add_modifier(Modifier::DIM);
            '.'
        } else {
            *raw_input
        };

        formatted_input.push(Span::styled(input.to_string(), style));
    }

    formatted_input.push(Span::raw("█"));

    while word_dex < idx_word_count
        && (word_dex < idx_input_count || *game_mode == GameMode::NonLatin)
    {
        // see if c is the same and new_char == true also
        if decide_break(
            game_mode,
            &indexable_word,
            &indexable_input,
            word_dex,
            last_input_char,
            new_char,
        ) {
            break;
        }

        formatted_word.push(Span::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));
        word_dex += 1;
    }

    let mut first = true;
    // Show the first error the user makes in the passage they are typing
    for word in indexable_word.iter().skip(word_dex).take(idx_word_count) {
        if first {
            if err {
                formatted_word.push(Span::styled(
                    word.to_string(),
                    Style::default().bg(Color::Red).fg(Color::White),
                ));
            } else {
                formatted_word.push(Span::styled(
                    word.to_string(),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ));
            }
            first = false;
            continue;
        }
        formatted_word.push(Span::raw(word.to_string()));
    }

    (formatted_word, formatted_input)
}

fn decide_break(
    game_mode: &GameMode,
    indexable_word: &[char],
    indexable_input: &[char],
    word_dex: usize,
    last_input_char: char,
    new_char: bool,
) -> bool {
    if *game_mode == GameMode::Latin {
        decide_break_latin(indexable_word, indexable_input, word_dex)
    } else {
        decide_break_nonlatin(indexable_word, indexable_input, last_input_char, new_char)
    }
}

fn decide_break_latin(indexable_word: &[char], indexable_input: &[char], word_dex: usize) -> bool {
    indexable_word[word_dex] != indexable_input[word_dex]
}

fn decide_break_nonlatin(
    indexable_word: &[char],
    indexable_input: &[char],
    last_input_char: char,
    new_char: bool,
) -> bool {
    !indexable_input.is_empty() || !new_char || vec![last_input_char] != indexable_word
}

fn get_fully_reformatted_texts<'a>(
    game_mode: &GameMode,
    words: &[&str],
    color: Color,
    end_string: &'a str,
    err: bool,
) -> FormattedTexts<'a> {
    let reformatted_complete_texts = (*words)
        .iter()
        .map(|word| {
            Span::styled(
                format!("{}{}", word, maybe_add_space(game_mode)),
                Style::default().fg(color),
            )
        })
        .collect();
    FormattedTexts {
        passage: reformatted_complete_texts,
        input: vec![Span::styled(
            end_string,
            Style::default().bg(color).fg(Color::White),
        )],
        error: err,
        complete: true,
    }
}

fn maybe_add_space(game_mode: &GameMode) -> &str {
    if *game_mode == GameMode::Latin {
        " "
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_formatted_words_correct() {
        // Test all letters are correct condition
        let test_word = "terminal-typeracer";
        let (formatted_word, formatted_input) =
            get_formatted_words(&GameMode::Latin, test_word, test_word, 'r', true);
        let properly_formatted_word: Vec<Span> = test_word
            .chars()
            .map(|it| Span::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        let mut properly_formatted_input: Vec<Span> = test_word
            .chars()
            .map(|it| Span::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        properly_formatted_input.push(Span::styled(" ", Style::default().bg(Color::Blue)));
        assert_eq!(formatted_word, properly_formatted_word);
        assert_eq!(formatted_input, properly_formatted_input);
    }

    #[test]
    fn test_get_formatted_words_err() {
        let test_word = "terminal-type";
        let test_input = "termimal-type";

        // There has to be a better way to do this
        let properly_formatted_word = vec![
            Span::styled("t", Style::default().fg(Color::Green)),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::styled("r", Style::default().fg(Color::Green)),
            Span::styled("m", Style::default().fg(Color::Green)),
            Span::styled("i", Style::default().fg(Color::Green)),
            Span::styled("n", Style::default().fg(Color::White).bg(Color::Red)),
            Span::raw("a"),
            Span::raw("l"),
            Span::raw("-"),
            Span::raw("t"),
            Span::raw("y"),
            Span::raw("p"),
            Span::raw("e"),
        ];

        let mut properly_formatted_input: Vec<Span> = test_input
            .chars()
            .map(|it| {
                Span::styled(
                    it.to_string(),
                    Style::default().fg(Color::White).bg(Color::Red),
                )
            })
            .collect();
        properly_formatted_input.push(Span::styled(" ", Style::default().bg(Color::Blue)));

        let (formatted_word, formatted_input) =
            get_formatted_words(&GameMode::Latin, test_word, test_input, 'e', true);

        assert_eq!(properly_formatted_word, formatted_word);
        assert_eq!(properly_formatted_input, formatted_input);
    }

    #[test]
    fn test_get_formatted_texts() {
        // Test that words are added in place to a set of formatted texts
        // Do not need to check correct vs incorrect. All we need to verify is that the formatted
        // texts are properly applied to the full set of formatted texts.
        let words = vec!["the", "quick", "brown", "fox"];
        let user_input = "bro";
        let current_word_idx = 2;
        let input_formatted_passage: Vec<Span> = vec![
            Span::styled("t", Style::default().fg(Color::Green)),
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::styled(" ", Style::default().fg(Color::Green)),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::styled("u", Style::default().fg(Color::Green)),
            Span::styled("i", Style::default().fg(Color::Green)),
            Span::styled("c", Style::default().fg(Color::Green)),
            Span::styled("k", Style::default().fg(Color::Green)),
            Span::styled(" ", Style::default().fg(Color::Green)),
            Span::raw("b"),
            Span::raw("r"),
            Span::raw("o"),
            Span::raw("w"),
            Span::raw("n"),
            Span::raw(" "),
            Span::raw("f"),
            Span::raw("o"),
            Span::raw("x"),
        ];

        let expected_formatted_passage: Vec<Span> = vec![
            Span::styled("t", Style::default().fg(Color::Green)),
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::styled(" ", Style::default().fg(Color::Green)),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::styled("u", Style::default().fg(Color::Green)),
            Span::styled("i", Style::default().fg(Color::Green)),
            Span::styled("c", Style::default().fg(Color::Green)),
            Span::styled("k", Style::default().fg(Color::Green)),
            Span::styled(" ", Style::default().fg(Color::Green)),
            Span::styled("b", Style::default().fg(Color::Green)),
            Span::styled("r", Style::default().fg(Color::Green)),
            Span::styled("o", Style::default().fg(Color::Green)),
            Span::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Span::raw("n"),
            Span::raw(" "),
            Span::raw("f"),
            Span::raw("o"),
            Span::raw("x"),
        ];

        let formatted_texts = get_formatted_texts(
            &GameMode::Latin,
            &words,
            user_input,
            current_word_idx,
            'x',
            true,
            input_formatted_passage,
        );

        assert_eq!(expected_formatted_passage, formatted_texts.passage);
        assert!(!formatted_texts.error);
    }

    #[test]
    fn test_get_formatted_line_mode() {
        // Test that words are added in place to a set of formatted texts
        // Do not need to check correct vs incorrect. All we need to verify is that the formatted
        // texts are properly applied to the full set of formatted texts.
        let words = vec!["the", "quick", "brown", "fox"];
        let user_input = "bro";
        let current_word_idx = 2;
        let input_formatted_passage: Vec<Span> = vec![
            Span::raw("b"),
            Span::raw("r"),
            Span::raw("o"),
            Span::raw("w"),
            Span::raw("n"),
            Span::raw(" "),
            Span::raw("f"),
            Span::raw("o"),
            Span::raw("x"),
        ];

        let expected_formatted_passage: Vec<Span> = vec![
            Span::styled("b", Style::default().fg(Color::Green)),
            Span::styled("r", Style::default().fg(Color::Green)),
            Span::styled("o", Style::default().fg(Color::Green)),
            Span::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Span::raw("n"),
            Span::raw(" "),
            Span::raw("f"),
            Span::raw("o"),
            Span::raw("x"),
        ];

        let formatted_texts = get_formatted_texts_line_mode(
            &GameMode::Latin,
            words[current_word_idx],
            user_input,
            'x',
            true,
            input_formatted_passage,
        );

        assert_eq!(expected_formatted_passage, formatted_texts.passage);
        assert!(!formatted_texts.error);
    }

    #[test]
    fn test_get_formatted_nonlatin_line_mode() {
        let words = vec!["你", "好", "你", "好"];
        let user_input = "";
        let current_word_idx = 1;

        let input_formatted_passage: Vec<Span> =
            vec![Span::raw("好"), Span::raw("你"), Span::raw("好")];

        let expected_formatted_passage: Vec<Span> = vec![
            Span::styled("好", Style::default().fg(Color::White).bg(Color::Blue)),
            Span::raw("你"),
            Span::raw("好"),
        ];

        let formatted_texts = get_formatted_texts_line_mode(
            &GameMode::NonLatin,
            words[current_word_idx],
            user_input,
            '你',
            true,
            input_formatted_passage,
        );

        assert_eq!(expected_formatted_passage, formatted_texts.passage);
        assert!(!formatted_texts.error);
    }

    #[test]
    fn get_formatted_texts_nonlatin() {
        let words = vec!["你", "好", "你", "好"];
        let user_input = "";
        let current_word_idx = 1;

        let input_formatted_passage: Vec<Span> =
            vec![Span::raw("好"), Span::raw("你"), Span::raw("好")];

        let expected_formatted_passage: Vec<Span> = vec![
            Span::styled("好", Style::default().fg(Color::White).bg(Color::Blue)),
            Span::raw("你"),
            Span::raw("好"),
        ];

        let formatted_texts = get_formatted_texts_line_mode(
            &GameMode::NonLatin,
            words[current_word_idx],
            user_input,
            '你',
            true,
            input_formatted_passage,
        );

        assert_eq!(expected_formatted_passage, formatted_texts.passage);
        assert!(!formatted_texts.error);
    }
}
