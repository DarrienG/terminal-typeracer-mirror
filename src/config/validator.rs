use std::io::{Error, ErrorKind};

use crate::config::RawTyperacerConfig;

/// Validates that the given config is valid.
/// If config is not valid, returns an err rather than the
/// user's config.
pub fn validate_config(config: RawTyperacerConfig) -> Result<RawTyperacerConfig, Error> {
    validate_lang_packs(&config)?;
    Ok(config)
}

/// Validate whether the lang_packs section is valid
/// Having no lang_packs config is valid, as is having
/// neither the whitelisted or blacklisted section filled out is valid.
/// Having both a blacklisted and whitelisted section is invalid.
fn validate_lang_packs(config: &RawTyperacerConfig) -> Result<(), Error> {
    match &config.lang_packs {
        None => Ok(()),
        Some(x) => {
            if x.blacklisted.is_some() && x.whitelisted.is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LangPacks;

    #[test]
    fn test_empty_config_ok() {
        assert!(validate_config(RawTyperacerConfig {
            lang_packs: None,
            display_settings: None,
            repo: None,
            repo_version: None,
            extra_repos: None,
            history_size: None,
        })
        .is_ok());
    }

    #[test]
    fn test_exclusive_blacklistwhitelist() {
        assert!(validate_config(RawTyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: Some(vec!["vrinda".to_string(), "punj".to_string()]),
                blacklisted: Some(vec!["tub".to_owned(), "golang".to_owned()]),
            }),
            display_settings: None,
            repo: None,
            repo_version: None,
            extra_repos: None,
            history_size: None,
        })
        .is_err());
    }

    #[test]
    fn test_blacklist_or_whitelist_ok() {
        assert!(validate_config(RawTyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: Some(vec!["vrinda".to_owned(), "punj".to_owned()]),
                blacklisted: None,
            }),
            display_settings: None,
            repo: None,
            repo_version: None,
            extra_repos: None,
            history_size: None,
        })
        .is_ok());

        assert!(validate_config(RawTyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: None,
                blacklisted: Some(vec!["tub".to_owned(), "golang".to_owned()]),
            }),
            display_settings: None,
            repo: None,
            repo_version: None,
            extra_repos: None,
            history_size: None,
        })
        .is_ok());
    }
}
