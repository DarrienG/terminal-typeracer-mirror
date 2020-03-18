use tui::widgets::Text;

use crate::game::split;

#[derive(PartialEq)]
pub enum GameMode {
    Latin,
    NonLatin,
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
        non_latin_word_complete(user_input, current_word)
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
