#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(option_take_if)]
#![feature(result_option_inspect)]
#![feature(test)]

use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};
use words::Word;

pub mod game;
pub mod guesser;
pub mod words;

#[derive(Deserialize)]
pub struct StringDictionary {
    pub valid: Vec<String>,
    pub answers: Vec<String>,
}

impl StringDictionary {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Dictionary<const N: usize> {
    pub valid: Vec<Word<N>>,
    pub answers: Vec<Word<N>>,
}

impl<const N: usize> Dictionary<N> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut result: Self = serde_json::from_reader(reader)?;

        result.valid.extend(result.answers.clone());
        result.valid.sort_unstable();
        result.answers.sort_unstable();

        Ok(result)
    }

    pub fn write_to_file<P: AsRef<Path>>(value: &Self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, value)?;
        Ok(())
    }
}
