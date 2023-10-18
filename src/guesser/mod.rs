use crate::words::Word;
use rayon::prelude::*;

pub mod naive_guesser;
pub use naive_guesser::NaiveGuesser;
pub mod bfs_bruteforce_guesser;
pub use bfs_bruteforce_guesser::BFSBruteforceGuesser;

pub trait Guesser<const N: usize>: Default + Send + Sync {
    fn rank_guess(
        guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
    ) -> f32;

    fn rank_guesses(
        &self,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
    ) -> Vec<(Word<N>, f32)> {
        let mut a: Vec<_> = valid_guesses
            .into_par_iter()
            .map(|guess| {
                (
                    *guess,
                    Self::rank_guess(guess, valid_guesses, possible_answers),
                )
            })
            .collect();

        a.as_parallel_slice_mut()
            .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

        a
    }
}
