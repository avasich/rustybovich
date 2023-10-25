use std::{
    cmp::{Eq, Ord},
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::{de::DeserializeOwned, Serialize};

mod words_1;

pub trait Family {
    type Word: WordTrait<F = Self>;
    type Pattern: PatternTrait<F = Self>;
}

pub type Pattern<F> = <F as Family>::Pattern;
pub type Word<F> = <F as Family>::Word;

pub trait WordTrait
where
    Self: Sized
        + Copy
        + Ord
        + Eq
        + Debug
        + Display
        + FromStr
        + Serialize
        + DeserializeOwned
        + Send
        + Sync,
    // <Self as FromStr>::Err: Debug,
{
    type F: Family;
    type Data;

    fn matches(&self, pattern: &Pattern<Self::F>, guess: &Self) -> bool;
    fn new(word: Self::Data) -> Self;
}

pub trait PatternTrait
where
    Self: Sized + Debug + Display + FromStr,
    // <Self as FromStr>::Err: Debug,
{
    type F: Family;
    type Data;

    fn new(pattern: Self::Data) -> Self;
    fn from_guess(guess: &Word<Self::F>, answer: &Word<Self::F>) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LetterType {
    Correct,
    Misplaced,
    Absent,
}

impl Display for LetterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use owo_colors::*;
        use LetterType::*;

        let style = match self {
            Correct => Style::new().green(),
            Misplaced => Style::new().yellow(),
            Absent => Style::new().default_color(),
        };
        write!(f, "{}", '■'.style(style))
    }
}
