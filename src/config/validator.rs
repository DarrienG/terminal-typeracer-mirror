use crate::config::TyperacerConfig;
use std::io::{Error, ErrorKind};

/// Validates that the given config is valid.
/// If config is not valid, returns an err rather than the
/// user's config.
pub fn validate_config(config: TyperacerConfig) -> Result<TyperacerConfig, Error> {
    validate_lang_packs(&config)?;
    Ok(config)
}

/// Validate whether the lang_packs section is valid
/// Having no lang_packs config is valid, as is having
/// neither the whitelisted or blacklisted section filled out is valid.
/// Having both a blacklisted and whitelisted section is invalid.
fn validate_lang_packs(config: &TyperacerConfig) -> Result<(), Error> {
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
        assert!(validate_config(TyperacerConfig {
            lang_packs: None,
            repo: None,
            repo_version: None,
        })
        .is_ok());
    }

    #[test]
    fn test_exclusive_blacklistwhitelist() {
        assert!(validate_config(TyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: Some(vec!["vrinda".to_string(), "punj".to_string()]),
                blacklisted: Some(vec!["tub".to_owned(), "golang".to_owned()]),
            }),
            repo: None,
            repo_version: None,
        })
        .is_err());
    }

    #[test]
    fn test_blacklist_or_whitelist_ok() {
        assert!(validate_config(TyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: Some(vec!["vrinda".to_owned(), "punj".to_owned()]),
                blacklisted: None,
            }),
            repo: None,
            repo_version: None,
        })
        .is_ok());

        assert!(validate_config(TyperacerConfig {
            lang_packs: Some(LangPacks {
                whitelisted: None,
                blacklisted: Some(vec!["tub".to_owned(), "golang".to_owned()]),
            }),
            repo: None,
            repo_version: None,
        })
        .is_ok());
    }
}
