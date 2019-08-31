use crate::config::TyperacerConfig;
use std::io::{Error, ErrorKind};

/// Validates that the given config is valid.
/// If config is not valid, returns an err rather than the
/// user's config.
pub fn validate_config(config: TyperacerConfig) -> Result<Option<TyperacerConfig>, Error> {
    let lang_pack_check = validate_lang_packs(&config);
    if lang_pack_check.is_err() {
        return Err(lang_pack_check.unwrap_err());
    }
    Ok(Some(config))
}

/// Validate whether the lang_packs section is valid
/// Having no lang_packs config is valid, as is having
/// neither the whitelisted or blacklisted section filled out is valid.
/// Having both a blacklisted and whitelisted section is invalid.
fn validate_lang_packs(config: &TyperacerConfig) -> Result<(), Error> {
    match &config.lang_packs {
        None => Ok(()),
        Some(x) => {
            if x.blacklisted.len() > 0 && x.whitelisted.len() > 0 {
                Err(Error::new(
                    ErrorKind::Other,
                    "Both blacklist and whitelist cannot be filled out",
                ))
            } else {
                Ok(())
            }
        }
    }
}
