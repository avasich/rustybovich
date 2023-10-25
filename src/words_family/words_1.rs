use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{Family, LetterType, Pattern, PatternTrait, Word, WordTrait};
use crate::{CollectArray, IteratorIntoArrayError};

pub struct Family1<const N: usize>;
impl<const N: usize> Family for Family1<N> {
    type Pattern = Pattern1<N>;
    type Word = Word1<N>;
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct Word1<const N: usize> {
    data: [char; N],
}

impl<const N: usize> Word1<N> {
    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.data.iter()
    }
}

impl<const N: usize> WordTrait for Word1<N> {
    type Data = [char; N];
    type F = Family1<N>;

    fn matches(&self, pattern: &Pattern<Self::F>, guess: &Self) -> bool {
        use LetterType::*;

        let mut answer: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = itertools::multizip((pattern.iter(), guess.iter(), answer.iter_mut()))
            .filter(|(pattern_letter, _, _)| **pattern_letter == Correct)
            .map(|(_, guess_letter, answer_letter)| {
                answer_letter.take_if(|letter| letter == guess_letter)
            })
            .all(|green_letter| green_letter.is_some());

        if !green_matches {
            return false;
        }

        std::iter::zip(pattern.iter(), guess.iter())
            .enumerate()
            .all(|(n, (pattern_letter, guess_letter))| match pattern_letter {
                Correct => true,
                Misplaced => answer
                    .iter()
                    .position(|answer_letter| {
                        answer_letter.is_some_and(|letter| letter == *guess_letter)
                    })
                    .and_then(|j| answer[j].take_if(|_| n != j))
                    .is_some(),
                Absent => answer
                    .iter()
                    .flatten()
                    .all(|answer_letter| *answer_letter != *guess_letter),
            })
    }

    fn new(word: Self::Data) -> Self {
        Self { data: word }
    }
}

impl<const N: usize> std::str::FromStr for Word1<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s.chars().collect_array()?;
        Ok(Self { data })
    }
}

impl<const N: usize> std::fmt::Display for Word1<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

#[derive(Debug)]
pub struct Pattern1<const N: usize> {
    data: [LetterType; N],
}

impl<const N: usize> Pattern1<N> {
    pub fn iter(&self) -> std::slice::Iter<'_, LetterType> {
        self.data.iter()
    }
}

impl<const N: usize> PatternTrait for Pattern1<N> {
    type Data = [LetterType; N];
    type F = Family1<N>;

    fn new(pattern: Self::Data) -> Self {
        Self { data: pattern }
    }

    fn from_guess(guess: &Word<Self::F>, answer: &Word<Self::F>) -> Self {
        use LetterType::*;

        let mut pattern = [Absent; N];
        let mut answer_letters: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        itertools::multizip((pattern.iter_mut(), guess.iter(), answer_letters.iter_mut()))
            .filter(|(_, guess_letter, ansewr_letter)| ansewr_letter.unwrap() == **guess_letter)
            .for_each(|(letter_type, _, ansewr_letter)| {
                ansewr_letter.take();
                *letter_type = Correct;
            });

        itertools::multizip((pattern.iter_mut(), guess.iter(), answer.data))
            .filter(|(_, guess_letter, answer_letter)| **guess_letter != *answer_letter)
            .for_each(|(letter_type, guess_letter, _)| {
                if let Some(answer_letter) = answer_letters.iter_mut().find(|answer_letter| {
                    answer_letter.is_some_and(|letter| letter == *guess_letter)
                }) {
                    answer_letter.take();
                    *letter_type = Misplaced;
                }
            });

        Self { data: pattern }
    }
}


impl<const N: usize> std::str::FromStr for Pattern1<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = s
            .chars()
            .map(|color| match color {
                'g' | '1' => LetterType::Correct,
                'y' | '2' => LetterType::Misplaced,
                _ => LetterType::Absent,
            })
            .collect_array()?;

        Ok(Self { data: pattern })
    }
}

impl<const N: usize> std::fmt::Display for Pattern1<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let res = self
            .iter()
            .try_fold(String::with_capacity(N), |mut out, c| {
                write!(out, "{c}").map(|_| out)
            })?;

        write!(f, "{res}")
    }
}
