use crate::words::{PatternCache, Word};
use rayon::prelude::*;

pub mod naive_guesser;
pub use naive_guesser::NaiveGuesser;

pub mod bfs_guesser;
pub use bfs_guesser::BfsGuesser;

pub trait Guesser<const N: usize>: Send + Sync {
    fn rank_guess(
        &self,
        guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: &PatternCache<N>,
    ) -> f32;

    fn rank_guesses(
        &self,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: &PatternCache<N>,
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
    Bfs(BfsGuesser),
}

impl GuesserWrapper {
    pub fn rank_guesses<const N: usize>(
        &self,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: &PatternCache<N>,
    ) -> Vec<(Word<N>, f32)> {
        match self {
            GuesserWrapper::Naive(g) => {
                g.rank_guesses(valid_guesses, possible_answers, pattern_cache)
            }
            GuesserWrapper::Bfs(g) => {
                g.rank_guesses(valid_guesses, possible_answers, pattern_cache)
            }
        }
    }
}

#[cfg(test)]
mod tests {}
