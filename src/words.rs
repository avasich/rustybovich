use std::{mem::MaybeUninit, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word<N> {
    pub fn new(word: [char; N]) -> Self {
        Self { word }
    }

    pub fn matches(&self, pattern: &Pattern<N>) -> bool {
        let mut w: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = pattern.iter().enumerate().all(|(i, c)| match *c {
            Colored::Green(c) if w[i].take() == Some(c) => true,
            Colored::Green(_) => false,
            _ => true,
        });

        if !green_matches {
            return false;
        }

        pattern.iter().enumerate().all(|(i, c)| match *c {
            Colored::Green(_) => true,
            Colored::Yellow(letter) if w[i] == Some(letter) => false,
            Colored::Yellow(letter) => match w.iter().position(|&other| other == Some(letter)) {
                Some(j) => {
                    w[j] = None;
                    true
                }
                None => false,
            },
            Colored::Gray(letter) => w.iter().all(|&other| other != Some(letter)),
        })
    }

    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.word.iter()
    }
}

impl<const N: usize> FromStr for Word<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word = s.chars().collect_array()?;
        Ok(Self { word })
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

    pub fn from_description(word: &str, descr: &str) -> Result<Pattern<N>, IteratorIntoArrayError> {
        let pattern = std::iter::zip(word.chars(), descr.chars())
            .map(|(letter, color)| match color {
                'g' | '1' => Colored::Green(letter),
                'y' | '2' => Colored::Yellow(letter),
                _ => Colored::Gray(letter),
            })
            .collect_array()?;

        Ok(Self { pattern })
    }

    pub fn from_guess(guess: &Word<N>, answer: &Word<N>) -> Self {
        let mut pattern: [_; N] = guess
            .iter()
            .cloned()
            .map(Colored::Gray)
            .collect_array()
            .unwrap();
        let mut word: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

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
                if let Some(j) = word.iter().position(|&c| c == Some(c1)) {
                    pattern[i] = Colored::Yellow(c1);
                    word[j] = None;
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
        use std::fmt::Write;

        let res = self.iter().fold(String::new(), |mut out, c| {
            let _ = write!(out, "{c}");
            out
        });

        write!(f, "{res}")
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

#[derive(Debug, PartialEq, Eq)]
pub struct IteratorIntoArrayError;

trait CollectArray<const N: usize>: std::iter::Iterator {
    fn collect_array(&mut self) -> Result<[Self::Item; N], IteratorIntoArrayError> {
        let mut array: [MaybeUninit<Self::Item>; N] = MaybeUninit::uninit_array();

        for array_ref in array.iter_mut() {
            let value = self.next().ok_or_else(|| IteratorIntoArrayError)?;
            array_ref.write(value);
        }

        Ok(unsafe { MaybeUninit::array_assume_init(array) })
    }
}

impl<const N: usize, I: std::iter::Iterator> CollectArray<N> for I {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_word() {
        assert_eq!(
            Word::<5>::from_str("crate"),
            Ok(Word::new(['c', 'r', 'a', 't', 'e']))
        );
        assert_eq!(
            Word::<5>::from_str("light"),
            Ok(Word::new(['l', 'i', 'g', 'h', 't']))
        );
        assert_eq!(
            Word::<5>::from_str("value"),
            Ok(Word::new(['v', 'a', 'l', 'u', 'e']))
        );
        assert_eq!(Word::<3>::from_str("foo"), Ok(Word::new(['f', 'o', 'o'])))
    }

    #[test]
    fn matches() {}
}
