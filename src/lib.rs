#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]

use std::mem::MaybeUninit;

use itertools::Itertools;
use std::iter::Iterator;

#[derive(Clone, Copy, Debug)]
pub struct Word<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word<N> {
    pub fn matches(&self, pattern: &Pattern<N>) -> bool {
        let mut w: [_; N] = self.iter().map(|&c| Some(c)).collect_array();

        for (i, c) in pattern.iter().enumerate() {
            match *c {
                Colored::Green(c1) => {
                    if w[i].take() != Some(c1) {
                        return false;
                    }
                }
                _ => continue,
            };
        }

        for (i, c) in pattern.iter().enumerate() {
            match *c {
                Colored::Green(_) => continue,
                Colored::Yellow(c1) => {
                    if w[i] == Some(c1) {
                        return false;
                    }

                    let pos = w.iter().position(|&c2| c2 == Some(c1));

                    match pos {
                        Some(idx) => {
                            w[idx] = None;
                        }
                        None => {
                            return false;
                        }
                    };
                }
                Colored::Gray(c1) => {
                    let pos = w.iter().position(|&c2| c2 == Some(c1));
                    if pos.is_some() {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.word.iter()
    }
}

impl<S: Into<String>, const N: usize> From<S> for Word<N> {
    fn from(value: S) -> Self {
        let word = value.into().chars().collect_array();
        Self { word }
    }
}

impl<const N: usize> std::fmt::Display for Word<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern<const N: usize> {
    pub pattern: [Colored; N],
}

impl<const N: usize> Pattern<N> {
    pub fn match_word(&self, word: &Word<N>) -> bool {
        word.matches(self)
    }

    pub fn filter_words(&self, words: &[Word<N>]) -> Vec<Word<N>> {
        words
            .iter()
            .filter(|word| self.match_word(word))
            .cloned()
            .collect()
    }

    pub fn from_description(word: &str, descr: &str) -> Self {
        let pattern = std::iter::zip(word.chars(), descr.chars())
            .map(|(letter, color)| match color {
                'g' => Colored::Green(letter),
                'y' => Colored::Yellow(letter),
                _ => Colored::Gray(letter),
            })
            .collect_array();

        Self { pattern }
    }

    pub fn from_guess(guess: &Word<N>, answer: &Word<N>) -> Self {
        let mut pattern: [_; N] = answer.iter().map(|&c| Colored::Gray(c)).collect_array();

        let mut word: [_; N] = answer.iter().map(|&c| Some(c)).collect_array();

        std::iter::zip(guess.word, answer.word)
            .enumerate()
            .filter(|&(_, (c1, c2))| c1 == c2)
            .for_each(|(i, (c1, _))| {
                pattern[i] = Colored::Green(c1);
                word[i] = None;
            });

        std::iter::zip(guess.word, answer.word)
            .enumerate()
            .filter(|&(_, (c1, c2))| c1 != c2)
            .for_each(|(i, (c1, _))| {
                let pos = word.iter().position(|&c| c == Some(c1));
                match pos {
                    Some(idx) => {
                        pattern[i] = Colored::Yellow(c1);
                        word[idx] = None;
                    }
                    None => {
                        pattern[i] = Colored::Gray(c1);
                    }
                }
            });

        Self { pattern }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Colored> {
        self.pattern.iter()
    }
}

impl<const N: usize> std::fmt::Display for Pattern<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = self.iter().map(|c| format!("{c}")).join("");

        write!(f, "{res}")
    }
}

impl<const N: usize> From<Vec<Colored>> for Pattern<N> {
    fn from(value: Vec<Colored>) -> Self {
        let word = value.try_into().unwrap();
        Self { pattern: word }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Colored {
    Green(char),
    Yellow(char),
    Gray(char),
}

impl Colored {
    pub fn value(&self) -> char {
        *match self {
            Colored::Green(c) => c,
            Colored::Yellow(c) => c,
            Colored::Gray(c) => c,
        }
    }
}

impl std::fmt::Display for Colored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        match self {
            Colored::Green(c) => write!(f, "{}", format!("{c}").green()),
            Colored::Yellow(c) => write!(f, "{}", format!("{c}").yellow()),
            Colored::Gray(c) => write!(f, "{}", format!("{c}").white()),
        }
    }
}

trait CollectArray<const N: usize>: Iterator {
    fn collect_array(&mut self) -> [Self::Item; N] {
        let mut array: [MaybeUninit<Self::Item>; N] = MaybeUninit::uninit_array();

        for array_ref in array.iter_mut() {
            let value = self.next().unwrap();
            array_ref.write(value);
        }

        unsafe { MaybeUninit::array_assume_init(array) }
    }
}
impl<const N: usize, I: Iterator> CollectArray<N> for I {}
