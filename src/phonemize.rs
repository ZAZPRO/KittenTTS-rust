use std::{collections::HashMap, path::Path, str::FromStr};

use cmudict_fast::{Cmudict, Rule};
use thiserror::Error;

const DICT: &str = include_str!("../model-files/cmudict.dict");

#[derive(Error, Debug, Clone)]
pub enum PhonemizerError {
    #[error("failed to load dictionary: {0}")]
    DictLoad(String),
}

#[derive(Debug)]
pub struct Phonemizer {
    dict: Cmudict,
    ipa: HashMap<&'static str, &'static str>,
}

fn get_ipa() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("AA", "ɑ"),
        ("AE", "æ"),
        ("AH", "ʌ"),
        ("AO", "ɔ"),
        ("AW", "aʊ"),
        ("AY", "aɪ"),
        ("EH", "ɛ"),
        ("ER", "ɚ"),
        ("EY", "eɪ"),
        ("IH", "ɪ"),
        ("IY", "i"),
        ("OW", "oʊ"),
        ("OY", "ɔɪ"),
        ("UH", "ʊ"),
        ("UW", "u"),
        ("B", "b"),
        ("CH", "tʃ"),
        ("D", "d"),
        ("DH", "ð"),
        ("F", "f"),
        ("G", "ɡ"),
        ("HH", "h"),
        ("JH", "dʒ"),
        ("K", "k"),
        ("L", "l"),
        ("M", "m"),
        ("N", "n"),
        ("NG", "ŋ"),
        ("P", "p"),
        ("R", "ɹ"),
        ("S", "s"),
        ("SH", "ʃ"),
        ("T", "t"),
        ("TH", "θ"),
        ("V", "v"),
        ("W", "w"),
        ("Y", "j"),
        ("Z", "z"),
        ("ZH", "ʒ"),
    ])
}

impl Phonemizer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PhonemizerError> {
        let dict = Cmudict::new(path).map_err(|e| PhonemizerError::DictLoad(e.to_string()))?;
        let ipa = get_ipa();
        Ok(Self { dict, ipa })
    }

    pub fn new() -> Result<Self, PhonemizerError> {
        let dict = Cmudict::from_str(DICT).map_err(|e| PhonemizerError::DictLoad(e.to_string()))?;
        let ipa = get_ipa();
        Ok(Self { dict, ipa })
    }

    pub fn phonemize(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        let upper = word.to_uppercase();
        let rules = self.dict.get(lower.as_str());
        let rule = if rules.is_none() {
            Rule::from_str(upper.as_str()).unwrap()
        } else {
            rules.unwrap()[0].clone()
        };

        let pronunciation = rule.pronunciation();
        let phonemized: String = if pronunciation.is_empty() {
            upper
        } else {
            pronunciation
                .iter()
                .map(|p| {
                    let key = p
                        .to_string()
                        .replace("0", "")
                        .replace("1", "")
                        .replace("2", "");

                    self.ipa[key.as_str()]
                })
                .collect()
        };

        phonemized
    }
}
