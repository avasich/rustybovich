use rayon::prelude::{ParallelIterator, *};

use crate::words_family::Family;

pub mod bfs_guesser;

pub trait Guesser<F: Family>
where
    Self: Send + Sync,
{
    fn rank_guess(
        &self,
        guess: &F::Word,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
    ) -> f32;

    fn rank_guesses(
        &self,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
    ) -> Vec<(F::Word, f32)> {
        let mut sorted_guesses: Vec<_> = valid_guesses
            .into_par_iter()
            .map(|guess| {
                (
                    *guess,
                    self.rank_guess(guess, valid_guesses, possible_answers),
                )
            })
            .collect();

        sorted_guesses
            .as_parallel_slice_mut()
            .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

        sorted_guesses
    }
}
