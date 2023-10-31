#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(option_take_if)]
#![feature(result_option_inspect)]
#![feature(test)]
#![feature(fn_traits)]

use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    mem::MaybeUninit,
    path::Path,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use words_family::Family;

pub mod game;
pub mod guesser_family;
pub mod words_family;

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
pub struct Dictionary<F: Family> {
    pub valid: Vec<F::Word>,
    pub answers: Vec<F::Word>,
}

impl<F: Family> Dictionary<F> {
    fn normalized(mut self) -> Self {
        self.answers.sort_unstable();
        self.answers.dedup();
        self.valid.extend(self.answers.clone());
        self.valid.sort_unstable();
        self.valid.dedup();

        self
    }

    pub fn read_from_json<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let result = serde_json::from_reader::<_, Self>(reader)?.normalized();

        Ok(result)
    }

    pub fn write_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn read_from_txt<P: AsRef<Path>>(
        path_valid: P,
        path_answers: P,
    ) -> Result<Self, Box<dyn Error>> {
        let valid = std::fs::read_to_string(path_valid)?
            .lines()
            .map(F::Word::from_str)
            .collect::<Result<_, _>>()?;
        let answers = std::fs::read_to_string(path_answers)?
            .lines()
            .map(F::Word::from_str)
            .collect::<Result<_, _>>()?;

        Ok(Self { valid, answers }.normalized())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IteratorIntoArrayError;

impl std::error::Error for IteratorIntoArrayError {}

impl std::fmt::Display for IteratorIntoArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "1")
    }
}

trait CollectArray<const N: usize>: std::iter::Iterator {
    fn collect_array(&mut self) -> Result<[Self::Item; N], IteratorIntoArrayError> {
        let mut array: [MaybeUninit<Self::Item>; N] = MaybeUninit::uninit_array();

        for array_ref in array.iter_mut() {
            let value = self.next().ok_or(IteratorIntoArrayError)?;
            array_ref.write(value);
        }

        if self.next().is_none() {
            Ok(unsafe { MaybeUninit::array_assume_init(array) })
        } else {
            Err(IteratorIntoArrayError)
        }
    }
}

impl<const N: usize, I: std::iter::Iterator> CollectArray<N> for I {}
