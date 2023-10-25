use std::{collections::HashMap, str::FromStr};

use itertools::Itertools;
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::LetterType;
use crate::{CollectArray, IteratorIntoArrayError};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct Word1<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word1<N> {
    pub fn new(word: [char; N]) -> Self {
        Self { word }
    }

    pub fn matches(&self, pattern: &Pattern1<N>) -> bool {
        let mut answer: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = std::iter::zip(pattern.iter(), answer.iter_mut())
            .filter(|(pattern_letter, _)| pattern_letter.letter_type == LetterType::Green)
            .all(|(pattern_letter, letter_option)| {
                letter_option
                    .take_if(|anser_letter| *anser_letter == pattern_letter.letter)
                    .is_some()
            });

        if !green_matches {
            return false;
        }

        pattern
            .iter()
            .enumerate()
            .all(|(i, &pattern_letter)| match pattern_letter.letter_type {
                LetterType::Green => true,
                LetterType::Yellow => answer
                    .iter()
                    .position(|&answer_letter| {
                        answer_letter.is_some_and(|letter| letter == pattern_letter.letter)
                    })
                    .and_then(|j| answer[j].take_if(|_| i != j))
                    .is_some(),
                LetterType::Gray => answer
                    .iter()
                    .flatten()
                    .all(|answer_letter| *answer_letter != pattern_letter.letter),
            })
    }

    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.word.iter()
    }
}

impl<const N: usize> FromStr for Word1<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word = s.chars().collect_array()?;
        Ok(Self { word })
    }
}

impl<const N: usize> std::fmt::Display for Word1<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

pub type PatternCache<'a, const N: usize> = HashMap<(&'a Word1<N>, &'a Word1<N>), Pattern1<N>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern1<const N: usize> {
    pub pattern: [PatternLetter; N],
}

impl<const N: usize> Pattern1<N> {
    pub fn new(pattern: [PatternLetter; N]) -> Self {
        Self { pattern }
    }

    pub fn from_description(
        pattern_colors: &str,
        guess: &str,
    ) -> Result<Pattern1<N>, IteratorIntoArrayError> {
        let pattern = std::iter::zip(guess.chars(), pattern_colors.chars())
            .map(|(letter, color)| match color {
                'g' | '1' => PatternLetter::green(letter),
                'y' | '2' => PatternLetter::yellow(letter),
                _ => PatternLetter::gray(letter),
            })
            .collect_array()?;

        Ok(Self { pattern })
    }

    pub fn from_guess(guess: &Word1<N>, answer: &Word1<N>) -> Self {
        let mut pattern: [_; N] = guess
            .iter()
            .cloned()
            .map(PatternLetter::gray)
            .collect_array()
            .unwrap();
        let mut answer_letters: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        std::iter::zip(pattern.iter_mut(), answer_letters.iter_mut())
            .filter(|(pattern_letter, answer_letter)| {
                answer_letter.unwrap() == pattern_letter.letter
            })
            .for_each(|(pattern_letter, answer_letter)| {
                answer_letter.take();
                pattern_letter.letter_type = LetterType::Green;
            });

        std::iter::zip(pattern.iter_mut(), answer.word)
            .filter(|(pattern_letter, answer_letter)| pattern_letter.letter != *answer_letter)
            .map(|(pattern_letter, _)| pattern_letter)
            .for_each(|pattern_letter| {
                if let Some(answer_letter) = answer_letters.iter_mut().find(|answer_letter| {
                    answer_letter.is_some_and(|letter| letter == pattern_letter.letter)
                }) {
                    answer_letter.take();
                    pattern_letter.letter_type = LetterType::Yellow;
                }
            });

        Self { pattern }
    }

    pub fn filter_words(&self, words: &[Word1<N>]) -> Vec<Word1<N>> {
        words
            .iter()
            .filter(|word| word.matches(self))
            .cloned()
            .collect()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, PatternLetter> {
        self.pattern.iter()
    }

    pub fn prepare_all<'a>(
        valid_guesses: &'a [Word1<N>],
        possible_answers: &'a [Word1<N>],
    ) -> PatternCache<'a, N> {
        valid_guesses
            .iter()
            .cartesian_product(possible_answers.iter())
            .map(|(guess, answer)| ((guess, answer), Pattern1::from_guess(guess, answer)))
            .collect()
    }
}

impl<const N: usize> std::fmt::Display for Pattern1<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let res = self.iter().fold(String::with_capacity(N), |mut out, c| {
            let _ = write!(out, "{c}");
            out
        });

        write!(f, "{res}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PatternLetter {
    letter_type: LetterType,
    letter: char,
}

impl PatternLetter {
    fn green(letter: char) -> Self {
        Self {
            letter_type: LetterType::Green,
            letter,
        }
    }

    fn yellow(letter: char) -> Self {
        Self {
            letter_type: LetterType::Yellow,
            letter,
        }
    }

    fn gray(letter: char) -> Self {
        Self {
            letter_type: LetterType::Gray,
            letter,
        }
    }
}

impl std::fmt::Display for PatternLetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        use LetterType::*;

        match self.letter_type {
            Green => write!(f, "{}", format!("{}", self.letter).green()),
            Yellow => write!(f, "{}", format!("{}", self.letter).yellow()),
            Gray => write!(f, "{}", format!("{}", self.letter).white()),
        }
    }
}

trait WrapLetter {
    fn green(self) -> PatternLetter;
    fn yellow(self) -> PatternLetter;
    fn gray(self) -> PatternLetter;
}

impl WrapLetter for char {
    fn green(self) -> PatternLetter {
        PatternLetter::green(self)
    }

    fn yellow(self) -> PatternLetter {
        PatternLetter::yellow(self)
    }

    fn gray(self) -> PatternLetter {
        PatternLetter::gray(self)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        assert_eq!(
            Word1::from_str("crate"),
            Ok(Word1::new(['c', 'r', 'a', 't', 'e']))
        );
        assert_eq!(
            Word1::from_str("light"),
            Ok(Word1::new(['l', 'i', 'g', 'h', 't']))
        );
        assert_eq!(
            Word1::from_str("value"),
            Ok(Word1::new(['v', 'a', 'l', 'u', 'e']))
        );
        assert_eq!(Word1::<3>::from_str("foo"), Ok(Word1::new(['f', 'o', 'o'])))
    }

    #[test]
    fn test_pattern_from_description() {
        assert_eq!(
            Pattern1::from_description("gg..y", "crate"),
            Ok(Pattern1::new([
                'c'.green(),
                'r'.green(),
                'a'.gray(),
                't'.gray(),
                'e'.yellow(),
            ]))
        );

        assert_eq!(
            Pattern1::from_description("yy..", "moon"),
            Ok(Pattern1::new([
                'm'.yellow(),
                'o'.yellow(),
                'o'.gray(),
                'n'.gray(),
            ]))
        );

        assert_eq!(
            Pattern1::from_description("y...g.", "maniac"),
            Ok(Pattern1::new([
                'm'.yellow(),
                'a'.gray(),
                'n'.gray(),
                'i'.gray(),
                'a'.green(),
                'c'.gray(),
            ]))
        );

        assert_eq!(
            Pattern1::from_description("g.y...", "gluers"),
            Ok(Pattern1::new([
                'g'.green(),
                'l'.gray(),
                'u'.yellow(),
                'e'.gray(),
                'r'.gray(),
                's'.gray(),
            ]))
        );

        assert_eq!(
            Pattern1::from_description("gyy", "bra"),
            Ok(Pattern1::new(['b'.green(), 'r'.yellow(), 'a'.yellow()]))
        );
    }

    pub fn pattern_and_words<const N: usize>(
        pattern: &str,
        guess: &str,
        answer: &str,
    ) -> (Pattern1<N>, Word1<N>, Word1<N>) {
        (
            Pattern1::from_description(pattern, guess).unwrap(),
            Word1::from_str(guess).unwrap(),
            Word1::from_str(answer).unwrap(),
        )
    }

    trait Matcher {
        fn is_matching<const N: usize>(pattern: &Pattern1<N>, answer: &Word1<N>) -> bool {
            answer.matches(pattern)
        }

        fn is_matching_str<const N: usize>(pattern: &str, guess: &str, answer: &str) -> bool {
            Self::is_matching::<N>(
                &Pattern1::from_description(pattern, guess).unwrap(),
                &answer.parse().unwrap(),
            )
        }
    }

    struct Matcher1;
    impl Matcher for Matcher1 {}

    fn test_matches_common<M: Matcher>() {
        assert!(M::is_matching_str::<3>("gyy", "bar", "bra"));
        assert!(M::is_matching_str::<5>("..ggg", "slate", "crate"));
        assert!(M::is_matching_str::<5>("gyyyy", "acbed", "abcde"));
        assert!(M::is_matching_str::<6>("g.y...", "gluers", "github"));
    }

    fn test_not_matches_common<M: Matcher>() {
        assert!(!M::is_matching_str::<3>("gy.", "baz", "bar"));
        assert!(!M::is_matching_str::<5>("....y", "crane", "aback"));
        assert!(!M::is_matching_str::<6>("g.y..g", "gluers", "github"));
        assert!(!M::is_matching_str::<6>("ggy...", "gluers", "github"));
        assert!(!M::is_matching_str::<6>("g.yy..", "gluers", "github"));
    }

    #[test]
    fn test_matches1() {
        test_matches_common::<Matcher1>();
    }

    #[test]
    fn test_not_matches1() {
        test_not_matches_common::<Matcher1>();
    }

    trait MakePattern {
        fn pattern<const N: usize>(guess: &Word1<N>, answer: &Word1<N>) -> Pattern1<N> {
            Pattern1::from_guess(guess, answer)
        }

        fn pattern_str<const N: usize>(guess: &str, answer: &str) -> Pattern1<N> {
            Self::pattern(&guess.parse().unwrap(), &answer.parse().unwrap())
        }
    }
    struct PatternBuilder1;

    impl MakePattern for PatternBuilder1 {}

    fn test_pattern_from_guess<P: MakePattern>() {
        assert_eq!(
            Pattern1::from_description("gyyyy", "acbed").unwrap(),
            P::pattern_str::<5>("acbed", "abcde")
        );

        assert_eq!(
            Pattern1::from_description("..ggg", "slate").unwrap(),
            P::pattern_str::<5>("slate", "crate")
        );

        assert_eq!(
            Pattern1::from_description("..y..", "brown").unwrap(),
            P::pattern_str::<5>("brown", "dicot")
        );

        assert_eq!(
            Pattern1::from_description("yg.g.", "thorp").unwrap(),
            P::pattern_str::<5>("thorp", "shirt")
        );

        assert_eq!(
            Pattern1::from_description(".....", "fghij").unwrap(),
            P::pattern_str::<5>("fghij", "abcde")
        );

        assert_eq!(
            Pattern1::from_description("ggggg", "abcde").unwrap(),
            P::pattern_str::<5>("abcde", "abcde")
        );

        assert_eq!(
            Pattern1::from_description("gggyy", "abcde").unwrap(),
            P::pattern_str::<5>("abcde", "abced")
        );
    }

    #[test]
    fn test_pattern_from_guess_1() {
        test_pattern_from_guess::<PatternBuilder1>();
    }
}
