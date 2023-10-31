use std::{
    cmp::{Eq, Ord},
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::{de::DeserializeOwned, Serialize};

pub mod words_1;

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
        + FromStrErrorHelper
        + Serialize
        + DeserializeOwned
        + Send
        + Sync,
{
    type F: Family<Word = Self>;
    type Data;

    fn matches(&self, pattern: &Pattern<Self::F>, guess: &Self) -> bool;
    fn new(word: Self::Data) -> Self;
}

pub trait PatternTrait
where
    Self: Sized + Debug + Display + FromStr + FromStrErrorHelper,
{
    type F: Family<Pattern = Self>;
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

pub trait FromStrErrorHelper
where
    Self: FromStr<Err = Self::FromStrErr>,
{
    type FromStrErr: std::error::Error + 'static;
}

impl<T> FromStrErrorHelper for T
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + 'static,
{
    type FromStrErr = <T as FromStr>::Err;
}
