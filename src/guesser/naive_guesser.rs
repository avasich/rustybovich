use crate::words::{Pattern, Word};

use super::Guesser;

#[derive(Clone, Copy)]
pub struct NaiveGuesser;

impl NaiveGuesser {}

impl<const N: usize> Guesser<N> for NaiveGuesser {
    fn rank_guess(
        guess: &Word<N>,
        _valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
    ) -> f32 {
        let matches: usize = possible_answers
            .iter()
            .map(|answer| {
                let pattern = Pattern::from_guess2(guess, answer);
                possible_answers
                    .iter()
                    .filter(|w| w.matches(&pattern))
                    .count()
            })
            .sum();

        (matches as f32) / (possible_answers.len() as f32)
    }
}
