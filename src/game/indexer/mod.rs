use unicode_segmentation::UnicodeSegmentation;

use crate::game::word_processing::GameMode;

/// Determine if two words are the same.
/// Check to see if the "input" is like the word. This is effectively
/// word.contains(input) but only if the first input.len characters are
/// the same. e.g. apple, ap => true, apple ppl => false
pub fn check_like_word(word: &str, input: &str) -> bool {
    let word_graphene = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();
    let input_graphene = UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>();

    if word_graphene.is_empty() {
        return true;
    }
    if input_graphene.len() > word_graphene.len() {
        return false;
    }

    word_graphene[..input_graphene.len()] == input_graphene[..]
}

/// Given a vector of word and the current index of the word the user is typing,
/// ["this", "is", "a", "vector"] and current_word_idx of 2,
/// return the index as if we were indexing the previous vector as a space
/// separated string to get the first character of the word the user is
/// currently on.
/// In this case, we would get 8 back.
/// "this is a vector"
/// ---------^
pub fn get_starting_idx(game_mode: &GameMode, words: &[&str], current_word_idx: usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for word in words.iter().take(current_word_idx) {
        passage_starting_idx += word.chars().count() + maybe_account_for_space(game_mode);
    }
    passage_starting_idx
}

/// Get the index of the letter as if words were a full string. Spaces counted.
pub fn get_trying_letter_idx(
    game_mode: &GameMode,
    words: &[&str],
    current_word_idx: usize,
    user_input: &str,
) -> usize {
    let starting_idx = get_starting_idx(game_mode, words, current_word_idx);

    let mut letter_on = 0;

    if user_input.is_empty() {
        return starting_idx;
    }

    let user_in_chars: Vec<char> = user_input.chars().collect();
    for c in words[current_word_idx].chars() {
        if letter_on == user_input.len() - 1 || user_in_chars[letter_on] != c {
            break;
        }
        letter_on += 1;
    }

    starting_idx + letter_on
}

pub fn get_maybe_decremented_idx(
    game_mode: &GameMode,
    user_has_error: bool,
    new_char: bool,
    current_word_idx: usize,
) -> usize {
    if *game_mode == GameMode::NonLatin && !user_has_error && current_word_idx > 0 && new_char {
        current_word_idx - 1
    } else {
        current_word_idx
    }
}

fn maybe_account_for_space(game_mode: &GameMode) -> usize {
    if *game_mode == GameMode::Latin {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_like_word() {
        // Normal case
        assert!(check_like_word("darrien", "darr"));

        // Full word
        assert!(check_like_word("darrien", "darrien"));

        // Input is longer than word to check
        assert!(!check_like_word("darrien", "darrienglasser.com"));

        // Case sensitivity
        assert!(!check_like_word("darrien", "Darrien"));

        // Non-matching non-latin/Multibyte characters
        assert!(!check_like_word("你好", "好你"));

        // Matching non-latin/Multibyte characters
        assert!(check_like_word("你好", "你好"));
    }

    #[test]
    fn test_get_starting_idx_latin() {
        let words = vec!["this", "is", "a", "vector"];
        assert!(get_starting_idx(&GameMode::Latin, &words, 2) == 8);
        assert!(get_starting_idx(&GameMode::Latin, &words, 0) == 0);
        assert!(get_starting_idx(&GameMode::Latin, &words, 1) == 5);
    }

    #[test]
    fn get_starting_idx_nonlatin() {
        let words = vec!["你", "好", "你", "好", "你", "好"];

        assert!(get_starting_idx(&GameMode::NonLatin, &words, 2) == 2);
        assert!(get_starting_idx(&GameMode::NonLatin, &words, 0) == 0);
        assert!(get_starting_idx(&GameMode::NonLatin, &words, 5) == 5);
    }

    #[test]
    fn test_get_trying_letter_index_good() {
        let words = vec!["the", "quick", "brown", "fox"];
        let current_word_idx = 1;
        let user_input = "qui";

        assert_eq!(
            get_trying_letter_idx(&GameMode::Latin, &words, current_word_idx, user_input),
            6
        );
    }

    #[test]
    fn test_get_trying_letter_index_bad() {
        let words = vec!["the", "quick", "brown", "fox"];
        let current_word_idx = 1;
        let user_input = "quisssssssssss";

        // Should be trying (and failing) the next letter
        assert_eq!(
            get_trying_letter_idx(&GameMode::Latin, &words, current_word_idx, user_input),
            7
        );
    }

    #[test]
    fn test_get_trying_letter_index_after_right() {
        let words = vec!["the", "quick", "brown", "fox"];
        let current_word_idx = 1;
        let user_input = "quisk";

        // Should not advance to the next character even though it's correct
        // because the previous is incorrect.
        assert_eq!(
            get_trying_letter_idx(&GameMode::Latin, &words, current_word_idx, user_input),
            7
        );
    }

    #[test]
    fn ensure_empty_input_is_valid() {
        let word = "quick";
        let user_input = "";

        assert!(check_like_word(word, user_input));
    }

    #[test]
    fn test_like_nonlatin() {
        let word = "你";
        let user_input = "你";

        assert!(check_like_word(word, user_input));
    }

    #[test]
    fn test_unlike_nonlatin() {
        let word = "你";
        let user_input = "好";

        assert!(!check_like_word(word, user_input));
    }
}
