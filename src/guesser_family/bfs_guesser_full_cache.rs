use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use itertools::Itertools;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::Guesser;
use crate::words_family::{Family, Pattern1, PatternTrait};

const LEN_TO_FIND: usize = 1;
const MAX_DEPTH: usize = 3;

pub struct BfsGuesserFullCache<F: Family>
where
    F::Pattern: PatternToU8,
    F::Word: Hash,
{
    patterns: Vec<u8>,
    size: usize,
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

    fn hash(&self, &guesses: &[Option<usize>; MAX_DEPTH]) -> u64 {
        // let mut bytes = [u8::MAX; 2 * MAX_DEPTH];

        // guesses
        //     .iter()
        //     .take_while(|guess| guess.is_some())
        //     .map(|guess| guess.unwrap())
        //     .enumerate()
        //     .for_each(|(i, guess)| {
        //         bytes[2 * i..2 * i + 2].copy_from_slice(&(guess as u16).to_le_bytes());
        //     });

        // bytes

        let n = guesses.iter().take_while(|guess| guess.is_some()).count();
        guesses
            .iter()
            .take_while(|guess| guess.is_some())
            .map(|guess| guess.unwrap())
            .enumerate()
            .map(|(i, g)| g * self.size.pow(n as u32 - i as u32))
            .sum::<usize>() as u64
    }

    fn rank_guess_against_answer(
        &self,
        guess: usize,
        answer: usize,
        valid_guesses: &[Option<usize>],
        possible_answers: &[usize],
        // cache: &mut HashMap<[Option<usize>; MAX_DEPTH], usize>,
        cache: &mut HashMap<u64, usize>,
    ) -> usize {
        let first_pattern = self.pattern(guess, answer);
        let mut initial_guesses = [None; MAX_DEPTH];
        initial_guesses[0] = Some(guess);

        let answers_left = possible_answers
            .iter()
            .filter(|&&possible_answer| self.pattern(guess, possible_answer) == first_pattern)
            .count();
        cache.insert(self.hash(&initial_guesses), answers_left);

        if answers_left <= LEN_TO_FIND {
            return 1;
        }

        let mut queue = VecDeque::new();
        queue.push_back(initial_guesses);

        while let Some(already_guessed) = queue.pop_front() {
            if already_guessed[MAX_DEPTH - 1].is_some() {
                return MAX_DEPTH;
            }

            let res = valid_guesses
                .iter()
                .filter(|&&guess| !already_guessed.contains(&guess))
                .map(|guess| {
                    let mut new_guesses = already_guessed;
                    new_guesses[MAX_DEPTH - 1] = *guess;
                    insertion_sort(&mut new_guesses, Option::gt);
                    new_guesses
                })
                .find_map(|guesses| match cache.get(&self.hash(&guesses)) {
                    Some(answers_left) => Some(*answers_left),
                    None => {
                        let answers_left = possible_answers
                            .iter()
                            .filter(|&&possible_answers| {
                                guesses
                                    .iter()
                                    .take_while(|guess| guess.is_some())
                                    .map(|guess| guess.unwrap())
                                    .all(|guess| {
                                        self.pattern(guess, answer)
                                            == self.pattern(guess, possible_answers)
                                    })
                            })
                            .count();

                        cache.insert(self.hash(&guesses), answers_left);

                        if answers_left <= LEN_TO_FIND {
                            let depth = guesses.iter().take_while(|guess| guess.is_some()).count();
                            Some(depth)
                        } else {
                            queue.push_back(guesses);
                            None
                        }
                    }
                });

            if let Some(res) = res {
                return res;
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

        let valid_guesses_indices: Vec<_> = valid_guesses
            .iter()
            .map(|word| *self.words_indices.get(word).unwrap())
            .map(Option::Some)
            .collect();
        let possible_answers_indices: Vec<_> = possible_answers
            .iter()
            .map(|word| *self.words_indices.get(word).unwrap())
            .collect();

        let total_answers = possible_answers.len();
        let counter = AtomicUsize::new(0);

        let ranks: Vec<_> = possible_answers_indices
            .par_iter()
            .map(|&answer_index| {
                let mut cache = HashMap::new();
                let res = valid_guesses_indices
                    .iter()
                    .flatten()
                    .map(|&guess_index| {
                        self.rank_guess_against_answer(
                            guess_index,
                            answer_index,
                            &valid_guesses_indices,
                            &possible_answers_indices,
                            &mut cache,
                        )
                    })
                    .collect_vec();

                if progress {
                    let completed = counter.fetch_add(1, Ordering::SeqCst);
                    print!("\ranswer {completed}/{total_answers}");
                    // println!();
                    // println!("{:?}", cache.keys().max());
                }
                res
            })
            .reduce_with(|mut res, rks| {
                res.iter_mut().zip(rks).for_each(|(r, rank)| *r += rank);
                res
            })
            .unwrap();

        if progress {
            println!();
        }

        valid_guesses
            .iter()
            .copied()
            .zip(ranks.iter().map(|&rk| rk as f32 / total_answers as f32))
            .sorted_unstable_by(|(_, rk1), (_, rk2)| rk1.partial_cmp(rk2).unwrap())
            .collect_vec()
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

pub fn insertion_sort<T, F>(arr: &mut [T], is_less: F)
where
    T: PartialOrd,
    F: Fn(&T, &T) -> bool,
{
    for i in 1..arr.len() {
        let mut j = i;
        while j > 0 && is_less(&arr[j], &arr[j - 1]) {
            unsafe { arr.swap_unchecked(j - 1, j) };
            j -= 1;
        }
    }
}
