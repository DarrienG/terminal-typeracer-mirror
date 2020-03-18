use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn to_words(passage: &str) -> Vec<&str> {
    // We're working with multi-width characters. The input is likely
    // a word per character rather than split on space.
    if is_wide_charater(passage) {
        return UnicodeSegmentation::graphemes(passage, true).collect::<Vec<&str>>();
    } else {
        passage.split(' ').collect()
    }
}

pub fn is_wide_charater(passage: &str) -> bool {
    passage
        .graphemes(true)
        .next()
        .expect("Unable to parse grapheme. Aborting.")
        .width()
        == 2
}
