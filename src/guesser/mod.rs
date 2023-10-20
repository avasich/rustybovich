use std::collections::HashMap;

use crate::words::{Pattern, Word};
use itertools::Itertools;
use rayon::prelude::*;

pub mod naive_guesser;
pub use naive_guesser::NaiveGuesser;

pub mod bfs_smart;
pub use bfs_smart::BFSSmartGuesser;

type PatternCache<'a, const N: usize> = HashMap<(&'a Word<N>, &'a Word<N>), Pattern<N>>;

pub fn prepare_patterns<'a, const N: usize>(
    guesses: &'a [Word<N>],
    answers: &'a [Word<N>],
) -> PatternCache<'a, N> {
    guesses
        .iter()
        .cartesian_product(answers.iter())
        .map(|(guess, answer)| ((guess, answer), Pattern::from_guess2(guess, answer)))
        .collect()
}

pub trait Guesser<const N: usize>: Send + Sync {
    fn rank_guess(
        &self,
        guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: Option<&PatternCache<N>>,
    ) -> f32;

    fn rank_guesses<'a>(
        &self,
        valid_guesses: &'a [Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: Option<&PatternCache<N>>,
    ) -> Vec<(Word<N>, f32)> {
        let mut sorted_guesses: Vec<_> = valid_guesses
            .into_par_iter()
            .map(|guess| {
                (
                    *guess,
                    self.rank_guess(guess, valid_guesses, possible_answers, pattern_cache),
                )
            })
            .collect();

        sorted_guesses
            .as_parallel_slice_mut()
            .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

        sorted_guesses
    }
}

pub enum GuesserWrapper {
    Naive(NaiveGuesser),
    BFSSmart(BFSSmartGuesser),
}

impl GuesserWrapper {
    pub fn rank_guesses<'a, const N: usize>(
        &self,
        valid_guesses: &'a [Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: Option<&PatternCache<N>>,
    ) -> Vec<(Word<N>, f32)> {
        match self {
            GuesserWrapper::Naive(g) => {
                g.rank_guesses(valid_guesses, possible_answers, pattern_cache)
            }
            GuesserWrapper::BFSSmart(g) => {
                g.rank_guesses(valid_guesses, possible_answers, pattern_cache)
            }
        }
    }
}

#[cfg(test)]
mod tests {}
