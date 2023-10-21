use std::{collections::HashMap, mem::MaybeUninit, str::FromStr};

use itertools::Itertools;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct Word<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word<N> {
    pub fn new(word: [char; N]) -> Self {
        Self { word }
    }

    pub fn matches(&self, pattern: &Pattern<N>) -> bool {
        let mut letters: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = std::iter::zip(letters.iter_mut(), pattern.iter())
            .filter(|(_, pattern_letter)| pattern_letter.letter_type == LetterType::Green)
            .all(|(letter_option, pattern_letter)| {
                letter_option
                    .take_if(|c| *c == pattern_letter.letter)
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
                LetterType::Yellow => letters
                    .iter()
                    .position(|&other| other.is_some_and(|c| c == pattern_letter.letter))
                    .and_then(|j| letters[j].take_if(|_| i != j))
                    .is_some(),
                LetterType::Gray => letters
                    .iter()
                    .all(|&other| other.map_or(true, |c| c != pattern_letter.letter)),
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

pub type PatternCache<'a, const N: usize> = HashMap<(&'a Word<N>, &'a Word<N>), Pattern<N>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern<const N: usize> {
    pub pattern: [PatternLetter; N],
}

impl<const N: usize> Pattern<N> {
    pub fn new(pattern: [PatternLetter; N]) -> Self {
        Self { pattern }
    }

    pub fn from_description(word: &str, descr: &str) -> Result<Pattern<N>, IteratorIntoArrayError> {
        let pattern = std::iter::zip(word.chars(), descr.chars())
            .map(|(letter, color)| match color {
                'g' | '1' => PatternLetter::green(letter),
                'y' | '2' => PatternLetter::yellow(letter),
                _ => PatternLetter::gray(letter),
            })
            .collect_array()?;

        Ok(Self { pattern })
    }

    pub fn from_guess(guess: &Word<N>, answer: &Word<N>) -> Self {
        let mut pattern: [_; N] = guess
            .iter()
            .cloned()
            .map(PatternLetter::gray)
            .collect_array()
            .unwrap();
        let mut answer_letters: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        std::iter::zip(pattern.iter_mut(), answer_letters.iter_mut())
            .filter(|(guess_letter, answer_letter)| answer_letter.unwrap() == guess_letter.letter)
            .for_each(|(guess_letter, answer_letter)| {
                answer_letter.take();
                guess_letter.letter_type = LetterType::Green;
            });

        std::iter::zip(pattern.iter_mut(), answer.word)
            .filter_map(|(guess_letter, answer_letter)| {
                (guess_letter.letter != answer_letter).then_some(guess_letter)
            })
            .for_each(|guess_letter| {
                let found = answer_letters.iter_mut().find(|answer_letter| {
                    answer_letter.is_some_and(|letter| letter == guess_letter.letter)
                });

                if let Some(answer_letter) = found {
                    answer_letter.take();
                    guess_letter.letter_type = LetterType::Yellow;
                }
            });

        Self { pattern }
    }

    pub fn from_guess2(guess: &Word<N>, answer: &Word<N>) -> Self {
        let mut pattern: [_; N] = guess
            .iter()
            .cloned()
            .map(PatternLetter::gray)
            .collect_array()
            .unwrap();
        let mut answer_letters: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        std::iter::zip(pattern.iter_mut(), answer_letters.iter_mut())
            .filter(|(guess_letter, answer_letter)| answer_letter.unwrap() == guess_letter.letter)
            .for_each(|(guess_letter, answer_letter)| {
                answer_letter.take();
                guess_letter.letter_type = LetterType::Green;
            });

        std::iter::zip(pattern.iter_mut(), answer.word)
            .filter(|(guess_letter, answer_letter)| guess_letter.letter != *answer_letter)
            .map(|(a, _)| a)
            .for_each(|guess_letter| {
                let found = answer_letters.iter_mut().find(|answer_letter| {
                    answer_letter.is_some_and(|letter| letter == guess_letter.letter)
                });

                if let Some(answer_letter) = found {
                    answer_letter.take();
                    guess_letter.letter_type = LetterType::Yellow;
                }
            });

        Self { pattern }
    }

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

    pub fn iter(&self) -> std::slice::Iter<'_, PatternLetter> {
        self.pattern.iter()
    }

    pub fn prepare_all<'a>(
        valid_guesses: &'a [Word<N>],
        possible_answers: &'a [Word<N>],
    ) -> PatternCache<'a, N> {
        valid_guesses
            .iter()
            .cartesian_product(possible_answers.iter())
            .map(|(guess, answer)| ((guess, answer), Pattern::from_guess(guess, answer)))
            .collect()
    }
}

impl<const N: usize> std::fmt::Display for Pattern<N> {
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
pub enum LetterType {
    Green,
    Yellow,
    Gray,
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        assert_eq!(
            Word::from_str("crate"),
            Ok(Word::new(['c', 'r', 'a', 't', 'e']))
        );
        assert_eq!(
            Word::from_str("light"),
            Ok(Word::new(['l', 'i', 'g', 'h', 't']))
        );
        assert_eq!(
            Word::from_str("value"),
            Ok(Word::new(['v', 'a', 'l', 'u', 'e']))
        );
        assert_eq!(Word::<3>::from_str("foo"), Ok(Word::new(['f', 'o', 'o'])))
    }

    #[test]
    fn test_pattern_from_description() {
        assert_eq!(
            Pattern::from_description("crate", "gg..y"),
            Ok(Pattern::new([
                'c'.green(),
                'r'.green(),
                'a'.gray(),
                't'.gray(),
                'e'.yellow(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("moon", "yy.."),
            Ok(Pattern::new([
                'm'.yellow(),
                'o'.yellow(),
                'o'.gray(),
                'n'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("maniac", "y...g."),
            Ok(Pattern::new([
                'm'.yellow(),
                'a'.gray(),
                'n'.gray(),
                'i'.gray(),
                'a'.green(),
                'c'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("gluers", "g.y..."),
            Ok(Pattern::new([
                'g'.green(),
                'l'.gray(),
                'u'.yellow(),
                'e'.gray(),
                'r'.gray(),
                's'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("bra", "gyy"),
            Ok(Pattern::new(['b'.green(), 'r'.yellow(), 'a'.yellow()]))
        );
    }

    pub fn word_and_pattern<const N: usize>(
        word: &str,
        pattern_word: &str,
        colors: &str,
    ) -> (Word<N>, Word<N>, Pattern<N>) {
        (
            Word::from_str(word).unwrap(),
            Word::from_str(pattern_word).unwrap(),
            Pattern::from_description(pattern_word, colors).unwrap(),
        )
    }

    trait MatcherBuilder {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Pattern<N>) -> bool;
    }

    struct Matcher1;

    impl MatcherBuilder for Matcher1 {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Pattern<N>) -> bool {
            Word::matches
        }
    }

    fn test_matches_common<B: MatcherBuilder>(matcher_builder: B) {
        let matcher = matcher_builder.build::<3>();

        let (word, _, pattern) = word_and_pattern("bar", "bra", "gyy");
        assert!(matcher(&word, &pattern));

        let matcher = matcher_builder.build::<5>();

        let (word, _, pattern) = word_and_pattern("crate", "slate", "..ggg");
        assert!(matcher(&word, &pattern));

        let (word, _, pattern) = word_and_pattern("abcde", "acbed", "gyyyy");
        assert!(matcher(&word, &pattern));

        let matcher = matcher_builder.build::<6>();

        let (word, _, pattern) = word_and_pattern("github", "gluers", "g.y...");
        assert!(matcher(&word, &pattern));
    }

    fn test_not_matches_common<B: MatcherBuilder>(matcher_builder: B) {
        let matcher = matcher_builder.build::<3>();

        let (word, _, pattern) = word_and_pattern("bar", "baz", "gy.");
        assert!(!matcher(&word, &pattern));

        let matcher = matcher_builder.build::<6>();

        let (word, _, pattern) = word_and_pattern("github", "gluers", "g.y..g");
        assert!(!matcher(&word, &pattern));

        let (word, _, pattern) = word_and_pattern("github", "gluers", "ggy...");
        assert!(!matcher(&word, &pattern));

        let (word, _, pattern) = word_and_pattern("github", "gluers", "g.yy..");
        assert!(!matcher(&word, &pattern));
    }

    #[test]
    fn test_matches1() {
        test_matches_common(Matcher1);
    }

    #[test]
    fn test_not_matches1() {
        test_not_matches_common(Matcher1);
    }

    trait PatternBuilder {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Word<N>) -> Pattern<N>;
    }

    struct PatternBuilder1;

    impl PatternBuilder for PatternBuilder1 {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Word<N>) -> Pattern<N> {
            Pattern::from_guess
        }
    }

    fn test_pattern_from_guess<B: PatternBuilder>(pattern_builder: B) {
        let pattern_from_guess = pattern_builder.build::<5>();

        let (answer, guess, pattern) = word_and_pattern("acbed", "abcde", "gyyyy");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("crate", "slate", "..ggg");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("dicot", "brown", "..y..");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("shirt", "thorp", "yg.g.");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("abcde", "fghij", ".....");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("abcde", "abcde", "ggggg");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);

        let (answer, guess, pattern) = word_and_pattern("abcde", "abced", "gggyy");
        assert_eq!(pattern_from_guess(&guess, &answer), pattern);
    }

    #[test]
    fn test_pattern_from_guess_2() {
        test_pattern_from_guess(PatternBuilder1);
    }
}
