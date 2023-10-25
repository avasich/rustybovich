pub mod words_1;
pub mod words_2;

pub use words_1::{Pattern1, PatternCache, Word1};
pub use words_2::{Pattern2, Word2};

// pub trait Word<const N: usize, Other = Self>
// where
//     Self: Display + Clone + Copy + Ord,
// {
//     type P: Pattern<N>;

//     fn matches(&self, pattern: &Self::P, guess: &Other) -> bool;
//     fn iter(&self) -> std::slice::Iter<'_, char>;
// }

// pub trait Pattern<const N: usize> {
//     fn from_guess<W>(guess: &W, answer: &W) -> Self;
//     fn iter(&self) -> std::slice::Iter<'_, LetterType>;
//     fn filter_words<W: Word<N>>(&self, guess: &W, words: &[W]) -> Vec<W>
//     where
//         W: Word<N, P = Self>,
//     {
//         words
//             .iter()
//             .filter(|word| word.matches(self, guess))
//             .cloned()
//             .collect()
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LetterType {
    Green,
    Yellow,
    Gray,
}
