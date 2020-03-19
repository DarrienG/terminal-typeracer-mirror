use tui::widgets::Text;

use crate::game::split;

#[derive(PartialEq)]
pub enum GameMode {
    Latin,
    NonLatin,
}

pub fn decide_game_end(
    game_mode: &GameMode,
    current_word_idx: usize,
    words: &[&str],
    user_input: &str,
) -> bool {
    if *game_mode == GameMode::Latin {
        decide_game_end_latin(current_word_idx, words, words[current_word_idx], user_input)
    } else {
        decide_game_nonlatin(words, current_word_idx, user_input)
    }
}

pub fn get_game_mode(raw_passage: &str) -> GameMode {
    if split::is_wide_character(raw_passage) {
        GameMode::NonLatin
    } else {
        GameMode::Latin
    }
}

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

pub fn get_updated_texts<'a>(
    game_mode: &GameMode,
    passage: Vec<Text<'a>>,
    current_word: &str,
) -> Vec<Text<'a>> {
    if *game_mode == GameMode::Latin {
        get_updated_latin_texts(passage, current_word)
    } else {
        get_updated_nonlatin_texts(passage)
    }
}

fn decide_game_end_latin(
    current_word_idx: usize,
    words: &[&str],
    current_word: &str,
    user_input: &str,
) -> bool {
    current_word_idx + 1 == words.len() && (current_word == user_input)
}

fn decide_game_nonlatin(words: &[&str], current_word_idx: usize, user_input: &str) -> bool {
    user_input.is_empty() && current_word_idx == words.len()
}

fn get_updated_latin_texts<'a>(passage: Vec<Text<'a>>, current_word: &str) -> Vec<Text<'a>> {
    passage[current_word.len() + 1..passage.len()].to_vec()
}

fn get_updated_nonlatin_texts<'a>(passage: Vec<Text<'a>>) -> Vec<Text<'a>> {
    passage[1..passage.len()].to_vec()
}

pub fn latin_word_complete(c: char, current_word: &str, user_input: &str) -> bool {
    c == ' ' && (current_word == user_input)
}

fn non_latin_word_complete(user_input: &str, current_word: &str) -> bool {
    user_input == current_word
}
