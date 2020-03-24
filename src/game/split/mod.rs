use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn to_words(passage: &str) -> Vec<&str> {
    // We're working with multi-width characters. The input is likely
    // a word per character rather than split on space.
    if is_wide_character(passage) {
        UnicodeSegmentation::graphemes(passage, true).collect::<Vec<&str>>()
    } else {
        passage.split(' ').collect()
    }
}

pub fn is_wide_character(passage: &str) -> bool {
    passage
        .graphemes(true)
        .next()
        .expect("Unable to parse grapheme. Aborting.")
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
}
