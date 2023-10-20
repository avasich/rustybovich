use crate::words::{Pattern, PatternCache, Word};

use super::Guesser;

pub struct NaiveGuesser;

impl NaiveGuesser {}

impl<const N: usize> Guesser<N> for NaiveGuesser {
    fn rank_guess(
        &self,
        guess: &Word<N>,
        _valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        _pattern_cache: &PatternCache<N>,
    ) -> f32 {
        let matches: usize = possible_answers
            .iter()
            .map(|answer| {
                let pattern = Pattern::from_guess(guess, answer);
                possible_answers
                    .iter()
                    .filter(|w| w.matches(&pattern))
                    .count()
            })
            .sum();

        (matches as f32) / (possible_answers.len() as f32)
    }
}
