use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    hash::Hash,
};

use priority_queue::PriorityQueue;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use super::Guesser;
use crate::words_family::{Family, Pattern1, PatternTrait};

pub struct BfsGuesserCachedPatterns<F: Family>
where
    F::Pattern: PatternToU8,
    F::Word: Hash,
{
    patterns: Vec<u8>,
    words_indices: HashMap<F::Word, usize>,
    size: usize,
}

pub trait PatternToU8
where
    Self: PatternTrait,
{
    fn as_u8(&self) -> u8;
}

impl<const N: usize> PatternToU8 for Pattern1<N> {
    fn as_u8(&self) -> u8 {
        self.iter().enumerate().fold(0, |acc, (i, l)| {
            use crate::words_family::LetterType::*;
            acc + match l {
                Correct => 0,
                Misplaced => 1,
                Absent => 2,
            } * 3u8.pow(i as u32)
        })
    }
}

impl<'a, F: Family> BfsGuesserCachedPatterns<F>
where
    F::Pattern: PatternToU8,
    F::Word: Hash,
{
    const LEN_TO_FIND: usize = 1;
    const MAX_DEPTH: usize = 3;

    pub fn new(valid: &'a [F::Word]) -> Self {
        let patterns = valid
            .iter()
            .flat_map(|guess| {
                valid
                    .iter()
                    .map(|answer| F::Pattern::from_guess(guess, answer).as_u8())
            })
            .collect();

        let words_indices: HashMap<_, _> = valid
            .iter()
            .enumerate()
            .map(|(i, word)| (*word, i))
            .collect();

        Self {
            patterns,
            words_indices,
            size: valid.len(),
        }
    }

    fn pattern(&self, guess_index: usize, answer_index: usize) -> u8 {
        self.patterns[guess_index * self.size + answer_index]
    }

    fn rank_guess_against_answer(
        &self,
        guess_index: usize,
        answer_index: usize,
        valid_guesses_indices: &[usize],
        possible_answers_indices: &[usize],
    ) -> usize {
        let valid_guesses = BTreeSet::from_iter(valid_guesses_indices.iter().copied());

        let first_pattern = self.pattern(guess_index, answer_index);
        let answers_left = possible_answers_indices
            .iter()
            .filter(|&&possible_answer_index| {
                self.pattern(guess_index, possible_answer_index) == first_pattern
            })
            .count();

        if answers_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut queue = PriorityQueue::new();
        queue.push(
            BTreeSet::from_iter(std::iter::once(guess_index)),
            P::new(1, answers_left),
        );

        while let Some((already_guessed, _p)) = queue.pop() {
            // too deep, we can do better
            if already_guessed.len() >= Self::MAX_DEPTH {
                return Self::MAX_DEPTH;
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

                let answers_left = possible_answers_indices
                    .iter()
                    .filter(|&&possible_answer_index| {
                        guesses.iter().all(|&guess_index| {
                            self.pattern(guess_index, answer_index)
                                == self.pattern(guess_index, possible_answer_index)
                        })
                    })
                    .count();

                let depth = guesses.len();

                if answers_left <= Self::LEN_TO_FIND {
                    return depth;
                }

                queue.push(guesses, P::new(depth, answers_left));
            }
        }

        panic!("out of guesses!");
    }

    fn rank_guess(
        &self,
        guess_index: usize,
        valid_guesses_indices: &[usize],
        possible_answers_indices: &[usize],
    ) -> f32 {
        let total: usize = possible_answers_indices
            .iter()
            .map(|&answer_index| {
                self.rank_guess_against_answer(
                    guess_index,
                    answer_index,
                    valid_guesses_indices,
                    possible_answers_indices,
                )
            })
            .sum();
        (total as f32) / (possible_answers_indices.len() as f32)
    }
}

impl<F: Family> Guesser<F> for BfsGuesserCachedPatterns<F>
where
    F::Word: Hash,
    F::Pattern: PatternToU8,
{
    fn rank_guesses(
        &self,
        valid_guesses: &[F::Word],
        possible_answers: &[F::Word],
        progress: bool,
    ) -> Vec<(F::Word, f32)> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let valid_guesses_indices: Vec<_> = valid_guesses
            .iter()
            .map(|word| *self.words_indices.get(word).unwrap())
            .collect();
        let possible_answers_indices: Vec<_> = possible_answers
            .iter()
            .map(|word| *self.words_indices.get(word).unwrap())
            .collect();

        let total_guesses = valid_guesses.len();
        let counter = AtomicUsize::new(0);

        let mut sorted_guesses: Vec<_> = valid_guesses
            .into_par_iter()
            .map(|guess| {
                let res = (
                    *guess,
                    self.rank_guess(
                        *self.words_indices.get(guess).unwrap(),
                        &valid_guesses_indices,
                        &possible_answers_indices,
                    ),
                );
                let completed = counter.fetch_add(1, Ordering::SeqCst);
                if progress {
                    print!("\rguess {completed}/{total_guesses} '{guess}'");
                }

                res
            })
            .collect();
        if progress {
            println!();
        }

        sorted_guesses
            .as_parallel_slice_mut()
            .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

        sorted_guesses
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