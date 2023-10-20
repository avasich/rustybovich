#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(option_take_if)]
#![feature(result_option_inspect)]
#![feature(test)]

use std::{error::Error, fs::File, io::BufReader, path::Path, str::FromStr};

use serde::Deserialize;
use words::Word;

pub mod game;
pub mod guesser;
pub mod words;

#[derive(Deserialize)]
struct DictionaryDeserialized {
    valid: Vec<String>,
    answers: Vec<String>,
}

pub struct Dictionary<const N: usize> {
    pub valid: Vec<Word<N>>,
    pub answers: Vec<Word<N>>,
}

impl<const N: usize> Dictionary<N> {
    fn read_file<P: AsRef<Path>>(path: P) -> Result<DictionaryDeserialized, Box<dyn Error>> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let DictionaryDeserialized { valid, answers } = Self::read_file(path)?;
        let mut valid: Vec<_> = valid
            .iter()
            .map(AsRef::as_ref)
            .map(Word::from_str)
            .collect::<Result<_, _>>()?;
        let mut answers: Vec<_> = answers
            .iter()
            .map(AsRef::as_ref)
            .map(Word::from_str)
            .collect::<Result<_, _>>()?;

        valid.extend(answers.clone());
        valid.sort_unstable();
        valid.dedup();
        answers.sort_unstable();

        Ok(Self { valid, answers })
    }
}
