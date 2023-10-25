use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::LetterType;
use crate::{CollectArray, IteratorIntoArrayError};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct Word2<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word2<N> {
    pub fn new(word: [char; N]) -> Self {
        Self { word }
    }

    pub fn matches(&self, pattern: &Pattern2<N>, guess: &Word2<N>) -> bool {
        let mut answer: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = itertools::multizip((pattern.iter(), guess.iter(), answer.iter_mut()))
            .filter(|(pattern_letter, _, _)| **pattern_letter == LetterType::Green)
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
                LetterType::Green => true,
                LetterType::Yellow => answer
                    .iter()
                    .position(|answer_letter| {
                        answer_letter.is_some_and(|letter| letter == *guess_letter)
                    })
                    .and_then(|j| answer[j].take_if(|_| n != j))
                    .is_some(),
                LetterType::Gray => answer
                    .iter()
                    .flatten()
                    .all(|answer_letter| *answer_letter != *guess_letter),
            })
    }

    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.word.iter()
    }
}

impl<const N: usize> FromStr for Word2<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word = s.chars().collect_array()?;
        Ok(Self { word })
    }
}

impl<const N: usize> std::fmt::Display for Word2<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern2<const N: usize> {
    pub pattern: [LetterType; N],
}

impl<const N: usize> Pattern2<N> {
    pub fn new(pattern: [LetterType; N]) -> Self {
        Self { pattern }
    }

    pub fn from_guess(guess: &Word2<N>, answer: &Word2<N>) -> Self {
        let mut pattern = [LetterType::Gray; N];
        let mut answer_letters: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        itertools::multizip((pattern.iter_mut(), guess.iter(), answer_letters.iter_mut()))
            .filter(|(_, guess_letter, ansewr_letter)| ansewr_letter.unwrap() == **guess_letter)
            .for_each(|(pattern_letter, _, ansewr_letter)| {
                ansewr_letter.take();
                *pattern_letter = LetterType::Green;
            });

        itertools::multizip((pattern.iter_mut(), guess.iter(), answer.word))
            .filter(|(_, guess_letter, answer_letter)| **guess_letter != *answer_letter)
            .for_each(|(pattern_letter, guess_letter, _)| {
                if let Some(answer_letter) = answer_letters.iter_mut().find(|answer_letter| {
                    answer_letter.is_some_and(|letter| letter == *guess_letter)
                }) {
                    answer_letter.take();
                    *pattern_letter = LetterType::Yellow;
                }
            });

        Self { pattern }
    }

    pub fn filter_words(&self, guess: &Word2<N>, words: &[Word2<N>]) -> Vec<Word2<N>> {
        words
            .iter()
            .filter(|word| word.matches(self, guess))
            .cloned()
            .collect()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, LetterType> {
        self.pattern.iter()
    }
}

impl<const N: usize> FromStr for Pattern2<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = s
            .chars()
            .map(|color| match color {
                'g' | '1' => LetterType::Green,
                'y' | '2' => LetterType::Yellow,
                _ => LetterType::Gray,
            })
            .collect_array()?;

        Ok(Self { pattern })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        assert_eq!(
            Word2::from_str("crate"),
            Ok(Word2::new(['c', 'r', 'a', 't', 'e']))
        );
        assert_eq!(
            Word2::from_str("light"),
            Ok(Word2::new(['l', 'i', 'g', 'h', 't']))
        );
        assert_eq!(
            Word2::from_str("value"),
            Ok(Word2::new(['v', 'a', 'l', 'u', 'e']))
        );
        assert_eq!(Word2::<3>::from_str("foo"), Ok(Word2::new(['f', 'o', 'o'])))
    }

    #[test]
    fn test_pattern_from_description() {
        use LetterType::*;

        assert_eq!(
            Pattern2::from_str("gg..y"),
            Ok(Pattern2::new([Green, Green, Gray, Gray, Yellow]))
        );

        assert_eq!(
            Pattern2::from_str("..yy"),
            Ok(Pattern2::new([Gray, Gray, Yellow, Yellow]))
        );

        assert_eq!(
            Pattern2::from_str("y...g."),
            Ok(Pattern2::new([Yellow, Gray, Gray, Gray, Green, Gray]))
        );

        assert_eq!(
            Pattern2::from_str("g.y..."),
            Ok(Pattern2::new([Green, Gray, Yellow, Gray, Gray, Gray]))
        );

        assert_eq!(
            Pattern2::from_str("gyy"),
            Ok(Pattern2::new([Green, Yellow, Yellow]))
        );
    }

    trait Matcher {
        fn is_matching<const N: usize>(
            pattern: &Pattern2<N>,
            guess: &Word2<N>,
            answer: &Word2<N>,
        ) -> bool {
            answer.matches(pattern, guess)
        }

        fn is_matching_str<const N: usize>(pattern: &str, guess: &str, answer: &str) -> bool {
            Self::is_matching::<N>(
                &pattern.parse().unwrap(),
                &guess.parse().unwrap(),
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
        fn pattern<const N: usize>(guess: &Word2<N>, answer: &Word2<N>) -> Pattern2<N> {
            Pattern2::from_guess(guess, answer)
        }

        fn pattern_str<const N: usize>(guess: &str, answer: &str) -> Pattern2<N> {
            Self::pattern(&guess.parse().unwrap(), &answer.parse().unwrap())
        }
    }

    struct PatternBuilder1;

    impl MakePattern for PatternBuilder1 {}

    fn test_pattern_from_guess<P: MakePattern>() {
        assert_eq!(
            Pattern2::from_str("gyyyy").unwrap(),
            P::pattern_str::<5>("acbed", "abcde")
        );
        assert_eq!(
            Pattern2::from_str("..ggg").unwrap(),
            P::pattern_str::<5>("slate", "crate")
        );

        assert_eq!(
            Pattern2::from_str("..y..").unwrap(),
            P::pattern_str::<5>("brown", "dicot")
        );

        assert_eq!(
            Pattern2::from_str("yg.g.").unwrap(),
            P::pattern_str::<5>("thorp", "shirt")
        );

        assert_eq!(
            Pattern2::from_str(".....").unwrap(),
            P::pattern_str::<5>("fghij", "abcde")
        );

        assert_eq!(
            Pattern2::from_str("ggggg").unwrap(),
            P::pattern_str::<5>("abcde", "abcde")
        );

        assert_eq!(
            Pattern2::from_str("gggyy").unwrap(),
            P::pattern_str::<5>("abcde", "abced")
        );
    }

    #[test]
    fn test_pattern_from_guess_2() {
        test_pattern_from_guess::<PatternBuilder1>();
    }
}
