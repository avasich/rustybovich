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

pub struct BfsGuesserFullCache<F: Family>
where
    F::Pattern: PatternToU8,
    F::Word: Hash,
{
    patterns: Vec<Vec<u8>>,
    words_indices: HashMap<F::Word, usize>,
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

impl<'a, F: Family> BfsGuesserFullCache<F>
where
    F::Pattern: PatternToU8,
    F::Word: Hash,
{
    const LEN_TO_FIND: usize = 1;
    const MAX_DEPTH: usize = 3;

    pub fn new(valid: &'a [F::Word]) -> Self {
        let patterns = valid
            .into_par_iter()
            .map(|guess| {
                valid
                    .iter()
                    .map(|answer| F::Pattern::from_guess(guess, answer).as_u8())
                    .collect()
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
        }
    }

    fn rank_guess_against_answer(
        &self,
        guess_index: usize,
        answer_index: usize,
        valid_guesses: &BTreeSet<usize>,
        possible_answers_indices: &[usize],
        cache: &mut HashMap<BTreeSet<usize>, usize>,
    ) -> usize {
        let first_pattern = self.patterns[guess_index][answer_index];

        let initial_set = BTreeSet::from_iter(std::iter::once(guess_index));

        let answers_left = possible_answers_indices
            .iter()
            .filter(|&&possible_answer_index| {
                self.patterns[guess_index][possible_answer_index] == first_pattern
            })
            .count();
        // cache.insert(initial_set.clone(), answers_left);

        if answers_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut queue = PriorityQueue::new();
        queue.push(initial_set, P::new(1, answers_left));

        while let Some((already_guessed, _p)) = queue.pop() {
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
                if let Some(res) = cache.get(&guesses) {
                    return *res;
                }

                let answers_left = possible_answers_indices
                    .iter()
                    .filter(|&&possible_answer_index| {
                        guesses
                            .iter()
                            .map(|&guess_index| &self.patterns[guess_index])
                            .all(|patterns| {
                                patterns[answer_index] == patterns[possible_answer_index]
                            })
                    })
                    .count();

                cache.insert(guesses.clone(), answers_left);

                let depth = guesses.len();

                if answers_left <= Self::LEN_TO_FIND {
                    return depth;
                }

                queue.push(guesses, P::new(depth, answers_left));
            }
        }

        panic!("out of guesses!");
    }
}

impl<F: Family> Guesser<F> for BfsGuesserFullCache<F>
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

        let valid_guesses_indices = BTreeSet::from_iter(
            valid_guesses
                .iter()
                .map(|word| *self.words_indices.get(word).unwrap()),
        );
        let possible_answers_indices: Vec<_> = possible_answers
            .iter()
            .map(|word| *self.words_indices.get(word).unwrap())
            .collect();

        let total_answers = possible_answers.len();
        let counter = AtomicUsize::new(0);

        let ranks: Vec<Vec<_>> = possible_answers_indices
            .clone()
            .into_par_iter()
            .map(|answer_index| {
                let mut cache = HashMap::new();
                let res = valid_guesses_indices
                    .iter()
                    .map(|&guess_index| {
                        self.rank_guess_against_answer(
                            guess_index,
                            answer_index,
                            &valid_guesses_indices,
                            &possible_answers_indices,
                            &mut cache,
                        )
                    })
                    .collect();

                if progress {
                    let completed = counter.fetch_add(1, Ordering::SeqCst);
                    print!("\ranswer {completed}/{total_answers}");
                }
                res
            })
            .collect();

        if progress {
            println!();
        }

        let mut sorted_guesses: Vec<_> = valid_guesses_indices
            .iter()
            .map(|&guess_index| {
                let rk = ranks.iter().map(|rank| rank[guess_index]).sum::<usize>() as f32
                    / ranks.len() as f32;
                (valid_guesses[guess_index], rk)
            })
            .collect();

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
