use std::cmp::Ordering;
use std::collections::BTreeSet;

use priority_queue::PriorityQueue;

use crate::words::{Pattern, Word};

use super::{Guesser, PatternCache};

#[derive(PartialEq, Eq)]
struct P {
    depth: usize,
    answers_left: usize,
}

impl P {
    fn new(depth: usize, answers_left: usize) -> Self {
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

pub struct BFSSmartGuesser;

impl BFSSmartGuesser {
    const LEN_TO_FIND: usize = 1;
    const MAX_DEPTH: usize = 3;

    fn rank_guess_against_answer<'a, const N: usize>(
        first_guess: &'a Word<N>,
        answer: &'a Word<N>,
        valid_guesses: &'a [Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: Option<&PatternCache<N>>,
    ) -> usize {
        let valid_guesses = BTreeSet::from_iter(valid_guesses.iter());

        let first_pattern = Pattern::from_guess2(first_guess, answer);
        let first_answeres_left = possible_answers
            .iter()
            .filter(|answer| answer.matches(&first_pattern))
            .count();

        if first_answeres_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut pq = PriorityQueue::new();
        pq.push(
            BTreeSet::from_iter(std::iter::once(first_guess)),
            P::new(1, first_answeres_left),
        );

        while let Some((aleady_guessed, p)) = pq.pop() {
            // too deep, we can do better
            if aleady_guessed.len() != p.depth {
                panic!("oh-oh");
            }

            if aleady_guessed.len() >= Self::MAX_DEPTH {
                return aleady_guessed.len();
            }

            let new_guesses = valid_guesses.difference(&aleady_guessed).map(|&guess| {
                let mut new_guesses = aleady_guessed.clone();
                new_guesses.insert(guess);
                new_guesses
            });

            for guesses in new_guesses {
                if pq.get_priority(&guesses).is_some() {
                    continue;
                }

                let depth = guesses.len();

                let answers_left = possible_answers
                    .iter()
                    .filter(|&possible_answer| {
                        guesses
                            .iter()
                            .map(|&guess| {
                                pattern_cache
                                    .and_then(|cache| cache.get(&(guess, answer)))
                                    .cloned()
                                    .unwrap_or_else(|| Pattern::from_guess2(guess, answer))
                            })
                            .all(|pattern| possible_answer.matches(&pattern))
                    })
                    .count();

                if answers_left <= Self::LEN_TO_FIND {
                    return depth;
                }

                pq.push(guesses, P::new(depth, answers_left));
            }
        }

        panic!("out of guesses!");
    }
}

impl<const N: usize> Guesser<N> for BFSSmartGuesser {
    fn rank_guess(
        &self,
        guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        pattern_cache: Option<&PatternCache<N>>,
    ) -> f32 {
        let total: usize = possible_answers
            .iter()
            .map(|answer| {
                Self::rank_guess_against_answer(
                    guess,
                    answer,
                    valid_guesses,
                    possible_answers,
                    pattern_cache,
                )
            })
            .sum();
        (total as f32) / (possible_answers.len() as f32)
    }
}
