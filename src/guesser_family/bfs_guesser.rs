use std::{cmp::Ordering, collections::BTreeSet, hash::Hash};

use priority_queue::PriorityQueue;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    guesser_family::Guesser,
    words_family::{Family, PatternTrait, WordTrait},
};

pub struct BfsGuesser;

impl<F: Family> Guesser<F> for BfsGuesser
where
    F::Word: Hash,
{
    fn rank_guesses(
        &self,
        valid_guesses: &[<F as Family>::Word],
        possible_answers: &[<F as Family>::Word],
    ) -> Vec<(<F as Family>::Word, f32)> {
        let mut sorted_guesses: Vec<_> = valid_guesses
            .into_par_iter()
            .map(|guess| {
                (
                    *guess,
                    self.rank_guess::<F>(guess, valid_guesses, possible_answers),
                )
            })
            .collect();

        sorted_guesses
            .as_parallel_slice_mut()
            .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

        sorted_guesses
    }
}

impl BfsGuesser {
    const LEN_TO_FIND: usize = 1;
    const MAX_DEPTH: usize = 3;

    fn rank_guess_against_answer<F: Family>(
        first_guess: &F::Word,
        answer: &F::Word,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
    ) -> usize
    where
        F::Word: Hash,
    {
        let valid_guesses = BTreeSet::from_iter(valid_guesses.iter());

        let first_pattern = F::Pattern::from_guess(first_guess, answer);
        let first_answers_left = possible_answers
            .iter()
            .filter(|answer| answer.matches(&first_pattern, first_guess))
            .count();

        if first_answers_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut queue = PriorityQueue::new();
        queue.push(
            BTreeSet::from_iter(std::iter::once(first_guess)),
            P::new(1, first_answers_left),
        );

        while let Some((already_guessed, p)) = queue.pop() {
            if already_guessed.len() != p.depth {
                panic!("oh-oh");
            }

            // too deep, we can do better
            if already_guessed.len() >= Self::MAX_DEPTH {
                return already_guessed.len();
            }

            let new_guesses = valid_guesses.difference(&already_guessed).map(|&guess| {
                let mut new_guesses = already_guessed.clone();
                new_guesses.insert(guess);
                new_guesses
            });

            for guesses in new_guesses {
                if queue.get_priority(&guesses).is_some() {
                    continue;
                }

                let depth = guesses.len();

                let answers_left = possible_answers
                    .iter()
                    .filter(|&possible_answer| {
                        guesses
                            .iter()
                            .map(|&guess| (F::Pattern::from_guess(guess, answer), guess))
                            .all(|(pattern, guess)| possible_answer.matches(&pattern, guess))
                    })
                    .count();

                if answers_left <= Self::LEN_TO_FIND {
                    return depth;
                }

                queue.push(guesses, P::new(depth, answers_left));
            }
        }

        panic!("out of guesses!");
    }

    fn rank_guess<F: Family>(
        &self,
        guess: &F::Word,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
    ) -> f32
    where
        F::Word: Hash,
    {
        let total: usize = possible_answers
            .iter()
            .map(|answer| {
                Self::rank_guess_against_answer::<F>(guess, answer, valid_guesses, possible_answers)
            })
            .sum();
        (total as f32) / (possible_answers.len() as f32)
    }
}

#[derive(PartialEq, Eq)]
pub struct P {
    depth: usize,
    answers_left: usize,
}

impl P {
    pub fn new(depth: usize, answers_left: usize) -> Self {
        Self {
            depth,
            answers_left,
        }
    }
}

impl PartialOrd for P {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for P {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.depth, other.answers_left).cmp(&(self.depth, self.answers_left))
    }
}
