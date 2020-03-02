use tui::style::{Color, Style};
use tui::widgets::Text;

use crate::game::indexer;

#[derive(Debug, Clone)]
pub struct FormattedTexts<'a> {
    pub passage: Vec<Text<'a>>,
    pub input: Vec<Text<'a>>,
    pub error: bool,
    pub complete: bool,
}

/// Should be the final formatting call.
/// Sets formatted texts fields to expected completion settings.
/// Reformats the entire passage from scratch in the case that the user is
/// running with display_settings.always_max=false.
/// If they are, they will only see the final word, but showing the whole
/// passage to them now that it is complete is a much better user experience.
pub fn get_reformatted_complete_texts<'a>(words: &[&str]) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(words, Color::Green, "COMPLETE", false)
}

pub fn get_reformatted_failed_texts<'a>(words: &[&str]) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(words, Color::Red, "FAIL", true)
}

/// Get fully formatted versions of the passage, and the user's input.
pub fn get_formatted_texts<'a>(
    words: &[&str],
    user_input: &str,
    current_word_idx: usize,
    mut formatted_passage: Vec<Text<'a>>,
) -> FormattedTexts<'a> {
    let (formatted_passage_word, formatted_input) =
        get_formatted_words(words[current_word_idx], user_input);

    let starting_idx = indexer::get_starting_idx(words, current_word_idx);

    formatted_passage[starting_idx..(formatted_passage_word.len() + starting_idx)]
        .clone_from_slice(&formatted_passage_word[..]);

    FormattedTexts {
        passage: formatted_passage,
        input: formatted_input,
        error: !indexer::check_like_word(words[current_word_idx], user_input),
        complete: false,
    }
}

pub fn get_formatted_texts_line_mode<'a>(
    current_word: &str,
    user_input: &str,
    mut formatted_passage: Vec<Text<'a>>,
) -> FormattedTexts<'a> {
    let (formatted_passage_word, formatted_input) = get_formatted_words(current_word, user_input);
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
/// On an erroroneous character:
/// - The first error character in the passage's word is highlighted with red and the rest unformatted.
/// - The entirety of the user's input is colored red.
///
/// Returns a tuple with the formatted version of the: word and the input.
fn get_formatted_words<'a>(word: &str, input: &str) -> (Vec<Text<'a>>, Vec<Text<'a>>) {
    let indexable_word: Vec<char> = word.chars().collect();
    let indexable_input: Vec<char> = input.chars().collect();
    let idx_word_count = indexable_word.len();
    let idx_input_count = indexable_input.len();

    let mut formatted_word: Vec<Text> = Vec::new();
    let mut formatted_input: Vec<Text> = Vec::new();
    let mut word_dex = 0;

    let err = !indexer::check_like_word(word, input);

    // Make all of the user's input white on red
    for input in indexable_input.iter() {
        if err {
            formatted_input.push(Text::styled(
                input.to_string(),
                Style::default().bg(Color::Red).fg(Color::White),
            ));
        } else {
            formatted_input.push(Text::styled(
                input.to_string(),
                Style::default().fg(Color::Green),
            ));
        }
    }

    formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));

    while word_dex < idx_word_count && word_dex < idx_input_count {
        if indexable_word[word_dex] != indexable_input[word_dex] {
            break;
        }

        formatted_word.push(Text::styled(
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
                formatted_word.push(Text::styled(
                    word.to_string(),
                    Style::default().bg(Color::Red).fg(Color::White),
                ));
            } else {
                formatted_word.push(Text::styled(
                    word.to_string(),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ));
            }
            first = false;
            continue;
        }
        formatted_word.push(Text::raw(word.to_string()));
    }

    (formatted_word, formatted_input)
}

fn get_fully_reformatted_texts<'a>(
    words: &[&str],
    color: Color,
    end_string: &'a str,
    err: bool,
) -> FormattedTexts<'a> {
    let reformatted_complete_texts = (*words)
        .iter()
        .map(|word| Text::styled(format!("{} ", word), Style::default().fg(color)))
        .collect();
    FormattedTexts {
        passage: reformatted_complete_texts,
        input: vec![Text::styled(
            end_string,
            Style::default().bg(color).fg(Color::White),
        )],
        error: err,
        complete: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_formatted_words_correct() {
        // Test all letters are correct condition
        let test_word = "terminal-typeracer";
        let (formatted_word, formatted_input) = get_formatted_words(test_word, test_word);
        let properly_formatted_word: Vec<Text> = test_word
            .chars()
            .map(|it| Text::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        let mut properly_formatted_input: Vec<Text> = test_word
            .chars()
            .map(|it| Text::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        properly_formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));
        assert!(formatted_word == properly_formatted_word);
        assert!(formatted_input == properly_formatted_input);
    }

    #[test]
    fn test_get_formatted_words_err() {
        let test_word = "terminal-type";
        let test_input = "termimal-type";

        // There has to be a better way to do this
        let properly_formatted_word = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("m", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("n", Style::default().fg(Color::White).bg(Color::Red)),
            Text::raw("a"),
            Text::raw("l"),
            Text::raw("-"),
            Text::raw("t"),
            Text::raw("y"),
            Text::raw("p"),
            Text::raw("e"),
        ];

        let mut properly_formatted_input: Vec<Text> = test_input
            .chars()
            .map(|it| {
                Text::styled(
                    it.to_string(),
                    Style::default().fg(Color::White).bg(Color::Red),
                )
            })
            .collect();
        properly_formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));

        let (formatted_word, formatted_input) = get_formatted_words(test_word, test_input);

        assert!(properly_formatted_word == formatted_word);
        assert!(properly_formatted_input == formatted_input);
    }

    #[test]
    fn test_get_formatted_texts() {
        // Test that words are added in place to a set of formatted texts
        // Do not need to check correct vs incorrect. All we need to verify is that the formatted
        // texts are properly applied to the full set of formatted texts.
        let words = vec!["the", "quick", "brown", "fox"];
        let user_input = "bro";
        let current_word_idx = 2;
        let input_formatted_passage: Vec<Text> = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("h", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("q", Style::default().fg(Color::Green)),
            Text::styled("u", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("c", Style::default().fg(Color::Green)),
            Text::styled("k", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::raw("b"),
            Text::raw("r"),
            Text::raw("o"),
            Text::raw("w"),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let expected_formatted_passage: Vec<Text> = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("h", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("q", Style::default().fg(Color::Green)),
            Text::styled("u", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("c", Style::default().fg(Color::Green)),
            Text::styled("k", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("b", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("o", Style::default().fg(Color::Green)),
            Text::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let formatted_texts = get_formatted_texts(
            &words,
            user_input,
            current_word_idx,
            input_formatted_passage,
        );

        assert!(expected_formatted_passage == formatted_texts.passage);
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
        let input_formatted_passage: Vec<Text> = vec![
            Text::raw("b"),
            Text::raw("r"),
            Text::raw("o"),
            Text::raw("w"),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let expected_formatted_passage: Vec<Text> = vec![
            Text::styled("b", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("o", Style::default().fg(Color::Green)),
            Text::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let formatted_texts = get_formatted_texts_line_mode(
            &words[current_word_idx],
            user_input,
            input_formatted_passage,
        );

        assert!(expected_formatted_passage == formatted_texts.passage);
        assert!(!formatted_texts.error);
    }
}
