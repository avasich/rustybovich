use super::Guesser;
use crate::words::{Pattern1, Word1};

pub struct NaiveGuesser;

impl NaiveGuesser {}

impl<const N: usize> Guesser<N> for NaiveGuesser {
    fn rank_guess(
        &self,
        guess: &Word1<N>,
        _valid_guesses: &[Word1<N>],
        possible_answers: &[Word1<N>],
        // _pattern_cache: &PatternCache<N>,
    ) -> f32 {
        let matches: usize = possible_answers
            .iter()
            .map(|answer| {
                let pattern = Pattern1::from_guess(guess, answer);
                possible_answers
                    .iter()
                    .filter(|possible_answer| possible_answer.matches(&pattern))
                    .count()
            })
            .sum();

        (matches as f32) / (possible_answers.len() as f32)
    }
}
