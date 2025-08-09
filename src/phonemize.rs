use std::{collections::HashMap, path::Path, str::FromStr};

use cmudict_fast::{Cmudict, Rule};
use thiserror::Error;

const DICT: &str = include_str!("../model-files/cmu.dict");

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
        ("AA1", "ɑː"),
        ("AA2", "ɑː"),
        ("AE", "æ"),
        ("AE1", "æ"),
        ("AE2", "æ"),
        ("AH", "ə"),
        ("AH1", "ʌ"),
        ("AH2", "ə"),
        ("AO", "ɔ"),
        ("AO1", "ɔː"),
        ("AO2", "ɔː"),
        ("AW", "aʊ"),
        ("AW1", "aʊ"),
        ("AW2", "aʊ"),
        ("AY", "aɪ"),
        ("AY1", "aɪ"),
        ("AY2", "aɪ"),
        ("EH", "ɛ"),
        ("EH1", "ɛ"),
        ("EH2", "ɛ"),
        ("ER", "ɝ"),
        ("ER1", "ɝː"),
        ("ER2", "ɝː"),
        ("EY", "eɪ"),
        ("EY1", "eɪ"),
        ("EY2", "eɪ"),
        ("IH", "ᵻ"),
        ("IH1", "ɪ"),
        ("IH2", "ɪ"),
        ("IY", "i"),
        ("IY1", "iː"),
        ("IY2", "iː"),
        ("OW", "oʊ"),
        ("OW1", "oʊ"),
        ("OW2", "oʊ"),
        ("OY", "ɔɪ"),
        ("OY1", "ɔɪ"),
        ("OY2", "ɔɪ"),
        ("UH", "ʊ"),
        ("UH1", "ʊ"),
        ("UH2", "ʊ"),
        ("UW", "u"),
        ("UW1", "uː"),
        ("UW2", "uː"),
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

    pub fn phonemize(&self, word: &str) -> Option<String> {
        let lower_case = word.to_lowercase();
        let upper_case = word.to_uppercase();

        let rules = self.dict.get(lower_case.as_str());
        let rule = if let Some(rule) = rules {
            rule[0].clone()
        } else {
            let rule_from_str = Rule::from_str(upper_case.as_str());
            match rule_from_str {
                Ok(rule) => rule,
                Err(_) => return None,
            }
        };

        let pronunciation = rule.pronunciation();
        let phonemized: String = if pronunciation.is_empty() {
            upper_case
        } else {
            pronunciation
                .iter()
                .map(|p| {
                    let key = p.to_string().replace("0", "");

                    self.ipa[key.as_str()]
                })
                .collect()
        };

        Some(phonemized)
    }
}
