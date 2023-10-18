use rayon::prelude::*;

use super::Guesser;
use crate::words::{Pattern, Word};
use std::collections::{HashSet, VecDeque};

#[derive(Default)]
pub struct BFSBruteforceGuesser;

impl BFSBruteforceGuesser {
    fn bar<const N: usize>(
        answer: &Word<N>,
        first_guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
    ) -> usize {
        let mut queue = VecDeque::new();
        let mut guessed = HashSet::new();
        guessed.insert(first_guess);
        queue.push_back(guessed);

        loop {
            match queue.pop_front() {
                Some(guessed) => {
                    let filtered_answers = guessed.iter().fold(
                        possible_answers.iter().collect::<Vec<_>>(),
                        |mut answers, guess| {
                            let pattern = Pattern::from_guess2(guess, answer);
                            answers.retain(|answer| answer.matches(&pattern));
                            answers
                        },
                    );

                    if filtered_answers.len() <= 3 {
                        return guessed.len();
                    }

                    if guessed.len() > 3 {
                        return guessed.len();
                    }

                    for guess in valid_guesses.iter() {
                        if guessed.contains(&guess) {
                            continue;
                        }

                        let mut guessed_next: HashSet<_> = guessed.iter().copied().collect();
                        guessed_next.insert(guess);

                        queue.push_back(guessed_next);
                    }
                }
                None => panic!("oh no, my bfs"),
            }
        }
    }
}

impl<const N: usize> Guesser<N> for BFSBruteforceGuesser {
    fn rank_guess(guess: &Word<N>, valid_guesses: &[Word<N>], possible_answers: &[Word<N>]) -> f32 {
        let total: usize = possible_answers
            .into_par_iter()
            .map(|answer| Self::bar(answer, guess, valid_guesses, possible_answers))
            .sum();
        (total as f32) / (possible_answers.len() as f32)
    }
}
