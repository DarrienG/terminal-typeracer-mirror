use std::io::Error;

mod game;
mod lang;
mod term_check;
mod dirs {
    pub mod setup_dirs;
}

fn main() -> Result<(), Error> {
    if !term_check::resolution_check().is_err() {
        if !lang::check_lang_pack() {
            let result = lang::retrieve_lang_pack();
            if result.is_err() {
                return result;
            }
        }
        while game::start_game() {}
    }
    Ok(())
}
