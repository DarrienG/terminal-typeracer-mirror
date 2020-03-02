/// Determine if two words are the same.
/// Check to see if the "input" is like the word. This is effectively
/// word.contains(input) but only if the first input.len characters are
/// the same. e.g. apple, ap => true, apple ppl => false
pub fn check_like_word(word: &str, input: &str) -> bool {
    if input.is_empty() {
        return true;
    }
    if input.len() > word.len() {
        return false;
    }

    check_word(&word[..input.len()], input)
}

fn check_word(word: &str, input: &str) -> bool {
    *word == *input
}

/// Given a vector of word and the current index of the word the user is typing,
/// ["this", "is", "a", "vector"] and current_word_idx of 2,
/// return the index as if we were indexing the previous vector as a space
/// separated string to get the first character of the word the user is
/// currently on.
/// In this case, we would get 8 back.
/// "this is a vector"
/// ---------^
pub fn get_starting_idx(words: &[&str], current_word_idx: usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for word in words.iter().take(current_word_idx) {
        passage_starting_idx += word.chars().count() + 1
    }
    passage_starting_idx
}

/// Get the index of the letter as if words were a full string. Spaces counted.
pub fn get_trying_letter_idx(words: &[&str], current_word_idx: usize, user_input: &str) -> usize {
    let starting_idx = get_starting_idx(words, current_word_idx);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_word() {
        assert!(check_word("darrien", "darrien"));
        assert!(!check_word("Darrien", "darrien"));
        assert!(!check_word("Darrien", "Glasser"));
    }

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
    }

    #[test]
    fn test_get_starting_idx() {
        let words = vec!["this", "is", "a", "vector"];
        assert!(get_starting_idx(&words, 2) == 8);
        assert!(get_starting_idx(&words, 0) == 0);
        assert!(get_starting_idx(&words, 1) == 5);
    }

    #[test]
    fn test_get_trying_letter_index_good() {
        let words = vec!["the", "quick", "brown", "fox"];
        let current_word_idx = 1;
        let user_input = "qui";

        assert_eq!(
            get_trying_letter_idx(&words, current_word_idx, user_input),
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
            get_trying_letter_idx(&words, current_word_idx, user_input),
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
            get_trying_letter_idx(&words, current_word_idx, user_input),
            7
        );
    }
}
