use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn to_words(passage: &str) -> Vec<&str> {
    // We're working with multi-width characters. The input is likely
    // a word per character rather than split on space.
    if is_wide_character(passage) {
        UnicodeSegmentation::graphemes(passage, true).collect::<Vec<&str>>()
    } else {
        passage.split_whitespace().collect()
    }
}

pub fn join_to_passage(words: &[&str]) -> String {
    match words.first() {
        Some(s) => {
            if is_wide_character(s) {
                words.join("")
            } else {
                words.join(" ")
            }
        }
        None => "".to_owned(),
    }
}

pub fn is_wide_character(passage: &str) -> bool {
    passage
        .graphemes(true)
        .next()
        .unwrap_or(" ") // provide some default just in case, should never be hit
        .width()
        == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_words_latin() {
        let passage = "the quick brown fox";
        let expected = ["the", "quick", "brown", "fox"];

        assert_eq!(to_words(passage), expected);
    }

    #[test]
    fn test_to_words_nonlatin() {
        let passage = "你好你好你好";
        let expected = ["你", "好", "你", "好", "你", "好"];

        assert_eq!(to_words(passage), expected);
    }

    #[test]
    fn test_join_to_passage_latin() {
        let split = ["the", "quick", "brown", "fox"];
        let expected = "the quick brown fox";

        assert_eq!(join_to_passage(&split), expected);
    }

    #[test]
    fn test_join_to_passage_nonlatin() {
        let split = ["你", "好", "你", "好", "你", "好"];
        let expected = "你好你好你好";

        assert_eq!(join_to_passage(&split), expected);
    }
}
