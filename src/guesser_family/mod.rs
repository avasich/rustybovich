use rayon::prelude::{ParallelIterator, *};

use crate::words_family::Family;

pub mod bfs_guesser;
pub mod bfs_guesser_cached_patterns;

pub trait Guesser<F: Family>
where
    Self: Send + Sync,
{
    fn rank_guesses(
        &self,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
    ) -> Vec<(F::Word, f32)>;
}
