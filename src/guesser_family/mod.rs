use crate::words_family::Family;

pub mod bfs_guesser;
pub mod bfs_guesser_cached_patterns;
pub mod bfs_guesser_full_cache;

pub use bfs_guesser::BfsGuesser;
pub use bfs_guesser_cached_patterns::BfsGuesserCachedPatterns;
pub use bfs_guesser_full_cache::BfsGuesserFullCache;

pub trait Guesser<F: Family>
where
    Self: Send + Sync,
{
    fn rank_guesses(
        &self,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
        progress: bool,
    ) -> Vec<(F::Word, f32)>;
}
