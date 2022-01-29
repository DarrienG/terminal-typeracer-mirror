use tui::text::Span;

use crate::game::split;

#[derive(PartialEq)]
pub enum GameMode {
    Latin,
    NonLatin,
}

/// This is effectively functionality for C-w. It removes the last word in a passage.
/// If there is a trailing space, it will remove that AND the word.
/// The goal is to mimic readline C-w as much as possible.
pub fn get_all_input_minus_last_word(input: &str) -> String {
    let words = split::to_words(input);
    if words.is_empty() {
        input.to_owned()
    } else {
        split::join_to_passage(&words[..words.len() - 1])
    }
}

/// Decide if game should end - i.e. the user has completed the passage.
/// The game has split functionality based on whether the passage is latin or nonlatin.
/// This is not determined by parsing, but by the `GameMode` passed in.
pub fn decide_game_end(
    game_mode: &GameMode,
    current_word_idx: usize,
    words: &[&str],
    user_input: &str,
) -> bool {
    if *game_mode == GameMode::Latin {
        decide_game_latin(current_word_idx, words, words[current_word_idx], user_input)
    } else {
        decide_game_nonlatin(words, current_word_idx, user_input)
    }
}

/// Retrieves `GameMode` based on the character encoding of the first character.
/// If the character is wide it is assumed to be NonLatin, otherwise we assume Latin.
pub fn get_game_mode(raw_passage: &str) -> GameMode {
    if split::is_wide_character(raw_passage) {
        GameMode::NonLatin
    } else {
        GameMode::Latin
    }
}

/// Decide if current word has been completed by user.
/// We trigger word completion for latin and non-latin words differently.
pub fn word_completed(game_mode: &GameMode, c: char, current_word: &str, user_input: &str) -> bool {
    if *game_mode == GameMode::Latin {
        latin_word_complete(c, current_word, user_input)
    } else {
        non_latin_word_complete(
            &format!("{}{}", user_input, c.encode_utf8(&mut [0; 4])),
            current_word,
        )
    }
}

/// Update texts in line mode for Latin and NonLatin texts.
pub fn get_updated_texts<'a>(
    game_mode: &GameMode,
    passage: Vec<Span<'a>>,
    current_word: &str,
) -> Vec<Span<'a>> {
    if *game_mode == GameMode::Latin {
        get_updated_latin_texts(passage, current_word)
    } else {
        get_updated_nonlatin_texts(passage)
    }
}

/// Decides if game is complete for latin words.
/// If we are but one character from completion and the current_word == user_input
/// then the user's got it.
fn decide_game_latin(
    current_word_idx: usize,
    words: &[&str],
    current_word: &str,
    user_input: &str,
) -> bool {
    current_word_idx + 1 == words.len() && (current_word == user_input)
}

/// Decides if the game is complete for NonLatin texts.
/// We first check if the user_input is empty. Since we read in characters
/// one at a time, and immediately determine if they are the same as the given
/// word because characters are words, if the user_input is empty, all they have
/// entered is the given character for the word.
/// We increment the index if the word is correct, so if the index is the same as
/// the word length, it means the user got the last character correctly.
fn decide_game_nonlatin(words: &[&str], current_word_idx: usize, user_input: &str) -> bool {
    user_input.is_empty() && current_word_idx == words.len()
}

/// For Latin texts, we want to get the length of the word + a space. We truncate
/// everything up to that point leaving the user with whatever is left.
fn get_updated_latin_texts<'a>(passage: Vec<Span<'a>>, current_word: &str) -> Vec<Span<'a>> {
    passage[current_word.len() + 1..passage.len()].to_vec()
}

/// For NonLatin texts, we want to truncate the the first character and nothing
/// more. In Chinese, this means one word == one char, so remove that word.
/// This approach will not scale with other languages, but we'll cross that
/// bridge when we come to it.
fn get_updated_nonlatin_texts(passage: Vec<Span>) -> Vec<Span> {
    passage[1..passage.len()].to_vec()
}

/// Determine if a Latin word is completed.
/// All Latin words are delineated with a space, so we check to see if the user
/// has entered <space> to decide if they have completed the word.
fn latin_word_complete(c: char, current_word: &str, user_input: &str) -> bool {
    c == ' ' && (current_word == user_input)
}

/// Determine if NonLatin word is completed.
/// In Chinese each word is a character, so we don't need to check if
/// there is a delimiter first.
fn non_latin_word_complete(user_input: &str, current_word: &str) -> bool {
    user_input == current_word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_end_latin() {
        let words = vec!["the", "quick", "brown", "fox"];
        let mut current_word_idx = 3;
        let mut user_input = "fox";

        assert!(decide_game_end(
            &GameMode::Latin,
            current_word_idx,
            &words,
            user_input
        ));

        current_word_idx = 1;

        assert!(!decide_game_end(
            &GameMode::Latin,
            current_word_idx,
            &words,
            user_input
        ));

        current_word_idx = 3;
        user_input = "bubbersarecoot";

        assert!(!decide_game_end(
            &GameMode::Latin,
            current_word_idx,
            &words,
            user_input
        ));
    }

    #[test]
    fn game_end_nonlatin() {
        let words = vec!["你", "好"];
        let mut current_word_idx = 2;
        let mut user_input = "";

        assert!(decide_game_end(
            &GameMode::NonLatin,
            current_word_idx,
            &words,
            user_input
        ));

        current_word_idx = 1;
        user_input = "好";

        assert!(!decide_game_end(
            &GameMode::NonLatin,
            current_word_idx,
            &words,
            user_input
        ));

        current_word_idx = 0;
        user_input = "好";

        assert!(!decide_game_end(
            &GameMode::NonLatin,
            current_word_idx,
            &words,
            user_input
        ));

        current_word_idx = 0;
        user_input = "bubbersarecoot";

        assert!(!decide_game_end(
            &GameMode::NonLatin,
            current_word_idx,
            &words,
            user_input
        ));
    }

    #[test]
    fn get_game_mode_interprets_latin() {
        let mut raw_passage = "the quick brown bubber";
        assert!(get_game_mode(raw_passage) == GameMode::Latin);

        raw_passage = "él zorro marrón rápido salta sobre el perro perezoso";
        assert!(get_game_mode(raw_passage) == GameMode::Latin);

        raw_passage = "πᾶν γράμμα";
        assert!(get_game_mode(raw_passage) == GameMode::Latin);

        raw_passage = "क्विक ब्राउन फ़ॉक्स";
        assert!(get_game_mode(raw_passage) == GameMode::Latin);
    }

    #[test]
    fn get_game_mode_interprets_nonlatin() {
        let mut raw_passage = "你好";
        assert!(get_game_mode(raw_passage) == GameMode::NonLatin);

        raw_passage = "速い茶色のキツネ";
        assert!(get_game_mode(raw_passage) == GameMode::NonLatin);
    }

    #[test]
    fn latin_word_completed() {
        let mut c = ' ';
        let mut user_input = "fox";
        let current_word = "fox";

        assert!(word_completed(
            &GameMode::Latin,
            c,
            current_word,
            user_input
        ));

        // do not trigger until user hits space
        c = 'j';
        assert!(!word_completed(
            &GameMode::Latin,
            c,
            current_word,
            user_input,
        ));

        c = ' ';
        user_input = "fo";
        assert!(!word_completed(
            &GameMode::Latin,
            c,
            current_word,
            user_input,
        ));

        // This will be correct if the user hits space next, but not yet.
        c = 'x';
        user_input = "fo";
        assert!(!word_completed(
            &GameMode::Latin,
            c,
            current_word,
            user_input,
        ));
    }

    #[test]
    fn nonlatin_word_completed() {
        let mut c = '好';
        let mut user_input = "";
        let current_word = "好";

        assert!(word_completed(
            &GameMode::NonLatin,
            c,
            current_word,
            user_input
        ));

        user_input = "你";
        assert!(!word_completed(
            &GameMode::NonLatin,
            c,
            current_word,
            user_input
        ));

        c = '你';
        user_input = "";
        assert!(!word_completed(
            &GameMode::NonLatin,
            c,
            current_word,
            user_input
        ));
    }

    #[test]
    fn get_updated_texts_latin() {
        let formatted_passage: Vec<Span> = vec![
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
        let current_word = "brown";

        let expected_formatted_passage: Vec<Span> =
            vec![Span::raw("f"), Span::raw("o"), Span::raw("x")];

        assert!(
            get_updated_texts(&GameMode::Latin, formatted_passage, current_word)
                == expected_formatted_passage
        );
    }

    #[test]
    fn get_updated_texts_nonlatin() {
        let formatted_passage: Vec<Span> = vec![
            Span::raw("亂"),
            Span::raw("數"),
            Span::raw("假"),
            Span::raw("文"),
            Span::raw("產"),
            Span::raw("生"),
            Span::raw("器"),
        ];
        let current_word = "亂";

        let expected_formatted_passage: Vec<Span> = vec![
            Span::raw("數"),
            Span::raw("假"),
            Span::raw("文"),
            Span::raw("產"),
            Span::raw("生"),
            Span::raw("器"),
        ];

        assert!(
            get_updated_texts(&GameMode::NonLatin, formatted_passage, current_word)
                == expected_formatted_passage
        );
    }

    #[test]
    fn removes_last_word_latin() {
        // note trailing space AND word should be removed
        let input = "this is a test ";
        let expected = "this is a";

        assert_eq!(get_all_input_minus_last_word(input), expected);

        // empty case
        let input = "";
        let expected = "";

        assert_eq!(get_all_input_minus_last_word(input), expected);

        let input = "él zorro marrón rápido salta sobre el perro";
        let expected = "él zorro marrón rápido salta sobre el";
        assert_eq!(get_all_input_minus_last_word(input), expected);
    }

    #[test]
    fn removes_last_word_nonlatin() {
        let input = "你好";
        let expected = "你";
        assert_eq!(get_all_input_minus_last_word(input), expected);

        let input = "速い茶色のキツネ";
        let expected = "速い茶色のキツ";
        assert_eq!(get_all_input_minus_last_word(input), expected);
    }
}
