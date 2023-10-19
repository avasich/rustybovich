use super::Guesser;

#[derive(Clone, Copy)]
pub struct BFSSmartGuesser;

impl<const N: usize> Guesser<N> for BFSSmartGuesser {
    fn rank_guess(
        guess: &crate::words::Word<N>,
        valid_guesses: &[crate::words::Word<N>],
        possible_answers: &[crate::words::Word<N>],
    ) -> f32 {
        todo!()
    }
}
