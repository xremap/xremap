use std::str::FromStr;

use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Deserializer};

// TODO: Use trait to allow only either `only` or `not`
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    #[serde(default, deserialize_with = "deserialize_matchers")]
    pub only: Option<Vec<ApplicationMatcher>>,
    #[serde(default, deserialize_with = "deserialize_matchers")]
    pub not: Option<Vec<ApplicationMatcher>>,
}

#[derive(Clone, Debug)]
pub enum ApplicationMatcher {
    // class.name
    Literal(String),
    // name
    Name(String),
    // /regex/
    Regex(Regex),
}

impl ApplicationMatcher {
    pub fn matches(&self, app: &str) -> bool {
        println!("matches self: '{:?}', app: '{:?}'", self, app);
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
    let v = deserialize_string_or_vec(deserializer)?;
    match v {
        None => Ok(None),
        Some(strings) => {
            let mut result: Vec<ApplicationMatcher> = vec![];
            for s in strings {
                result.push(ApplicationMatcher::from_str(&s).map_err(serde::de::Error::custom)?);
            }
            Ok(Some(result))
        }
    }
}

pub fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    let vec = match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Vec(vec) => vec,
        StringOrVec::String(string) => vec![string],
    };
    Ok(Some(vec))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
