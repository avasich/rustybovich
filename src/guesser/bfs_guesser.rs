use std::collections::BTreeSet;
use std::{cmp::Ordering, collections::VecDeque};

use priority_queue::PriorityQueue;

use crate::words::{Pattern, PatternCache, Word};

use super::Guesser;

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

pub struct BfsGuesser;

impl BfsGuesser {
    const LEN_TO_FIND: usize = 1;
    const MAX_DEPTH: usize = 3;

    #[allow(dead_code)]
    fn rank_guess_against_answer<const N: usize>(
        first_guess: &Word<N>,
        answer: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        _pattern_cache: &PatternCache<N>,
    ) -> usize {
        let valid_guesses = BTreeSet::from_iter(valid_guesses.iter());

        let first_pattern = Pattern::from_guess(first_guess, answer);
        let first_answeres_left = possible_answers
            .iter()
            .filter(|answer| answer.matches(&first_pattern))
            .count();

        if first_answeres_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut queue = PriorityQueue::new();
        queue.push(
            BTreeSet::from_iter(std::iter::once(first_guess)),
            P::new(1, first_answeres_left),
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
                            .map(|&guess| Pattern::from_guess(guess, answer))
                            .all(|pattern| possible_answer.matches(&pattern))
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

    fn rank_guess_against_answer_deque<const N: usize>(
        first_guess: &Word<N>,
        answer: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        _pattern_cache: &PatternCache<N>,
    ) -> usize {
        let valid_guesses = BTreeSet::from_iter(valid_guesses.iter());

        let first_pattern = Pattern::from_guess(first_guess, answer);
        let first_answeres_left = possible_answers
            .iter()
            .filter(|answer| answer.matches(&first_pattern))
            .count();

        if first_answeres_left <= Self::LEN_TO_FIND {
            return 1;
        }

        let mut deque = VecDeque::new();
        deque.push_back(BTreeSet::from_iter(std::iter::once(first_guess)));

        // let mut cache = HashMap::new();

        while let Some(already_guessed) = deque.pop_front() {
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
                if deque.contains(&guesses) {
                    continue;
                }

                let depth = guesses.len();

                let answers_left = possible_answers
                    .iter()
                    .filter(|&possible_answer| {
                        guesses
                            .iter()
                            .map(|&guess| Pattern::from_guess(guess, answer))
                            .all(|pattern| possible_answer.matches(&pattern))
                        // cache
                        // .entry((guess, answer))
                        // .or_insert_with(|| Pattern::from_guess(guess, answer))
                        // .match_word(possible_answer)
                    })
                    .count();

                if answers_left <= Self::LEN_TO_FIND {
                    return depth;
                }

                deque.push_back(guesses);
            }
        }

        panic!("out of guesses!");
    }
}

impl<const N: usize> Guesser<N> for BfsGuesser {
    fn rank_guess(
        &self,
        guess: &Word<N>,
        valid_guesses: &[Word<N>],
        possible_answers: &[Word<N>],
        _pattern_cache: &PatternCache<N>,
    ) -> f32 {
        let total: usize = possible_answers
            .iter()
            .map(|answer| {
                // Self::rank_guess_against_answer(
                //     guess,
                //     answer,
                //     valid_guesses,
                //     possible_answers,
                //     pattern_cache,
                // )
                Self::rank_guess_against_answer_deque(
                    guess,
                    answer,
                    valid_guesses,
                    possible_answers,
                    _pattern_cache,
                )
            })
            .sum();
        (total as f32) / (possible_answers.len() as f32)
    }
}
