use crate::config::deserializers::VecOrSingle;
use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

// Used for both application and window-title matching.
//
// Matching semantics:
// - `only`: matches if ANY of the matchers matches (OR).
// - `not`:  matches if NONE of the matchers matches.
// - Both set: both conditions must hold (AND).
// - Neither set: never matches (keymap only applies with an explicit matcher).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationMatch {
    #[serde(default, deserialize_with = "deserialize_matchers")]
    only: Option<Vec<ApplicationMatcher>>,
    #[serde(default, deserialize_with = "deserialize_matchers")]
    not: Option<Vec<ApplicationMatcher>>,
}

impl ApplicationMatch {
    pub fn matches(&self, app: &str) -> bool {
        if let Some(only) = &self.only {
            if !only.iter().any(|m| m.matches(app)) {
                return false;
            }
        }
        if let Some(not) = &self.not {
            if not.iter().any(|m| m.matches(app)) {
                return false;
            }
        }
        self.only.is_some() || self.not.is_some()
    }
}

#[derive(Clone, Debug)]
enum ApplicationMatcher {
    // class.name
    Literal(String),
    // name
    Name(String),
    // /regex/
    Regex(Regex),
}

impl ApplicationMatcher {
    pub fn matches(&self, app: &str) -> bool {
        match &self {
            ApplicationMatcher::Literal(s) => s == app,
            ApplicationMatcher::Name(s) => {
                if let Some(pos) = app.rfind('.') {
                    s == &app[(pos + 1)..]
                } else {
                    s == app
                }
            }
            ApplicationMatcher::Regex(r) => r.is_match(app),
        }
    }
}

impl FromStr for ApplicationMatcher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [b'/', ..] => Ok(ApplicationMatcher::Regex(Regex::new(&slash_unescape(s)?)?)),
            _ => {
                if s.find('.').is_some() {
                    Ok(ApplicationMatcher::Literal(s.to_owned()))
                } else {
                    Ok(ApplicationMatcher::Name(s.to_owned()))
                }
            }
        }
    }
}

fn slash_unescape(s: &str) -> anyhow::Result<String> {
    let mut result = String::with_capacity(s.len());
    let mut escaping = false;
    let mut finished = false;
    for c in s.chars().skip(1) {
        if finished {
            return Err(anyhow!("Unexpected trailing string after closing / in application name regex"));
        }
        if escaping {
            escaping = false;
            if c != '/' {
                result.push('\\');
            }
            result.push(c);
            continue;
        }
        match c {
            '/' => finished = true,
            '\\' => escaping = true,
            _ => result.push(c),
        }
    }
    if !finished {
        return Err(anyhow!("Missing closing / in application name regex"));
    }
    Ok(result)
}

fn deserialize_matchers<'de, D>(deserializer: D) -> Result<Option<Vec<ApplicationMatcher>>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = VecOrSingle::deserialize(deserializer)?.into_vec();
    let mut result = vec![];
    for s in strings {
        result.push(ApplicationMatcher::from_str(&s).map_err(serde::de::Error::custom)?);
    }
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let matcher = ApplicationMatcher::from_str("class.name").unwrap();
        assert!(matcher.matches("class.name"));
        assert!(!matcher.matches("class"));
        assert!(!matcher.matches("name"));
    }

    #[test]
    fn test_name_match() {
        let matcher = ApplicationMatcher::from_str("name").unwrap();
        assert!(matcher.matches("class.name"));
        assert!(matcher.matches("name"));
        assert!(!matcher.matches("name.foo"));
        assert!(!matcher.matches("name-and-more"));
    }

    #[test]
    fn test_literal_application_name_matcher() {
        let matcher = ApplicationMatcher::from_str(r"Minecraft").unwrap();
        assert!(matcher.matches("Minecraft"), "Failed to match exact Minecraft");
        assert!(!matcher.matches("Minecraft Launcher"), "Literal matcher should not be used as substring");
    }

    #[test]
    fn test_regex_application_name_matcher() {
        let matcher = ApplicationMatcher::from_str(r"/^Minecraft\*? \d+\.\d+(\.\d+)?$/").unwrap();
        assert!(matcher.matches(r"Minecraft 1.19.2"), "Failed to match Minecraft 1.19.2 using regex");
        assert!(matcher.matches(r"Minecraft* 1.19"), "Failed to match Minecraft* 1.19 using regex");
        assert!(matcher.matches(r"Minecraft* 1.19.2"), "Failed to match Minecraft* 1.19.2 using regex");
    }

    #[test]
    fn test_regex_unescape_application_name_matcher() {
        let matcher = ApplicationMatcher::from_str(r"/^\/$/").unwrap();
        assert!(matcher.matches(r"/"), "Failed to match single slash using regex");
    }

    #[test]
    fn test_unescape_slash_correct_regex() {
        let given = r"/^Mine\d\/craft\\/";
        let got = slash_unescape(given).unwrap();
        assert_eq!(r"^Mine\d/craft\\", got);
    }

    #[test]
    fn test_unescape_slash_missing_closing_slash() {
        let given = r"/^Minecraft\/";
        let got = slash_unescape(given).unwrap_err();
        assert_eq!("Missing closing / in application name regex", got.to_string());
    }

    #[test]
    fn test_unescape_slash_excessive_string_after_closing() {
        let given = r"/^Minecraft/i";
        let got = slash_unescape(given).unwrap_err();
        assert_eq!("Unexpected trailing string after closing / in application name regex", got.to_string());
    }

    fn match_config(only: Option<Vec<&str>>, not: Option<Vec<&str>>, app: &str) -> bool {
        let parse = |items: Vec<&str>| -> Vec<ApplicationMatcher> {
            items.iter().map(|s| ApplicationMatcher::from_str(s).unwrap()).collect()
        };
        ApplicationMatch {
            only: only.map(|v| parse(v)),
            not: not.map(|v| parse(v)),
        }
        .matches(app)
    }

    #[test]
    fn test_match_only() {
        assert!(match_config(Some(vec!["emacs"]), None, "emacs"));
        assert!(!match_config(Some(vec!["emacs"]), None, "firefox"));
    }

    #[test]
    fn test_match_not() {
        assert!(match_config(None, Some(vec!["firefox"]), "emacs"));
        assert!(!match_config(None, Some(vec!["firefox"]), "firefox"));
    }

    #[test]
    fn test_match_only_and_not_combined() {
        // only AND not must both hold
        assert!(match_config(Some(vec!["emacs"]), Some(vec![r"/-pi\\*/"]), "emacs"));
        assert!(!match_config(Some(vec!["emacs"]), Some(vec![r"/-pi\\*/"]), "firefox"));
    }
}
