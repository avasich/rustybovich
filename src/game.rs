#![allow(dead_code)]
use std::str::FromStr;

use crate::words::{IteratorIntoArrayError, Pattern, Word};
use colored::*;
use itertools::Itertools;
use rayon::prelude::*;

fn expected_left<const N: usize>(guess: &Word<N>, possible_answers: &[Word<N>]) -> f64 {
    let matches: usize = possible_answers
        .iter()
        .map(|answer| {
            let pattern = Pattern::from_guess(guess, answer);
            possible_answers
                .iter()
                .filter(|w| w.matches(&pattern))
                .count()
        })
        .sum();

    (matches as f64) / (possible_answers.len() as f64)
}

fn sort_guesses<const N: usize>(
    valid_guesses: &[Word<N>],
    possible_answers: &[Word<N>],
) -> Vec<(Word<N>, f64)> {
    let mut a: Vec<_> = valid_guesses
        .into_par_iter()
        .map(|guess| (*guess, expected_left(guess, possible_answers)))
        .collect();

    a.as_parallel_slice_mut()
        .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

    a
}

fn slow_deep_rank_answer<const N: usize>(
    guess: &Word<N>,
    answer: &Word<N>,
    words_left: &[Word<N>],
) -> f64 {
    if guess == answer {
        return 1.0;
    }
    let pattern = Pattern::from_guess(guess, answer);
    let matching_words = pattern.filter_words(words_left);

    if matching_words.len() == words_left.len() {
        panic!("wtf")
    }

    let rank = matching_words.iter().fold(0.0, |acc, word| {
        acc + slow_deep_rank_answer(word, answer, &matching_words)
    });

    rank
}

fn slow_deep_rank_all_answers<const N: usize>(
    guess: &Word<N>,
    possible_answers: &[Word<N>],
) -> f64 {
    let rank = possible_answers.iter().fold(0.0, |arr, answer| {
        arr + slow_deep_rank_answer(guess, answer, possible_answers)
    });

    rank / possible_answers.len() as f64
}

fn slow_deep_sort_guesses<const N: usize>(
    guesses: &[Word<N>],
    possible_answers: &[Word<N>],
) -> Vec<(Word<N>, f64)> {
    let mut a: Vec<_> = guesses
        .into_par_iter()
        .map(|guess| (*guess, slow_deep_rank_all_answers(guess, possible_answers)))
        .collect();

    a.as_parallel_slice_mut()
        .sort_unstable_by(|(_w1, n1), (_w2, n2)| n1.partial_cmp(n2).unwrap());

    a
}

#[derive(PartialEq, Clone)]
enum Mode {
    Hard,
    Normal,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Hard => "hard".red(),
            Mode::Normal => "norm".green(),
        };
        write!(f, "{s}")
    }
}

#[derive(PartialEq, Clone)]
enum Command {
    Next,
    Exit,
    Clear,
    Show,
    ShowGuesses,
    Mode(Mode),
    Undo,
    Guess,
    PatternDescription { word: String, colors: String },
}

pub struct Game<const N: usize> {
    valid: Vec<Word<N>>,
    answers: Vec<Word<N>>,
}

impl<const N: usize> Game<N> {
    pub fn new<S: AsRef<str>>(valid: &[S], answers: &[S]) -> Result<Self, IteratorIntoArrayError> {
        let mut valid: Vec<_> = valid
            .iter()
            .map(S::as_ref)
            .map(Word::from_str)
            .collect::<Result<_, _>>()?;
        let mut answers: Vec<_> = answers
            .iter()
            .map(S::as_ref)
            .map(Word::from_str)
            .collect::<Result<_, _>>()?;

        valid.extend(answers.clone());
        valid.sort_unstable();
        valid.dedup();
        answers.sort_unstable();

        Ok(Self { valid, answers })
    }

    fn game(&self) -> Command {
        let mut possible_answers = self.answers.clone();
        let mut possible_answers_bk = possible_answers.clone();
        let mut guesses = vec![];
        let mut mode = Mode::Normal;

        loop {
            let command = loop {
                println!(
                    "[{mode}] | {} words left | {}:",
                    possible_answers.len(),
                    "pattern or command".cyan()
                );
                match self.read_command() {
                    c @ (Command::Next | Command::Exit) => return c,
                    Command::Clear => {
                        let _ = std::process::Command::new("clear").status().unwrap();
                    }
                    Command::Show => {
                        println!(
                            "{} words left:\n[{}]",
                            possible_answers.len(),
                            Self::word_list_to_string(&possible_answers)
                        );
                    }
                    Command::ShowGuesses => Self::show_guesses(&guesses, &possible_answers, 10),
                    Command::Mode(m) => mode = m,
                    Command::Undo => possible_answers = possible_answers_bk.clone(),
                    c @ Command::Guess => break c,
                    c @ Command::PatternDescription { word: _, colors: _ } => break c,
                }
            };

            match command {
                Command::Guess => {
                    let valid_guesses = match mode {
                        Mode::Hard => &possible_answers,
                        Mode::Normal => &self.valid,
                    };
                    guesses = sort_guesses(valid_guesses, &possible_answers);
                    // guesses = slow_deep_sort_guesses(guess_from, &words_left);
                    //
                    Self::show_guesses(&guesses, &possible_answers, 10);

                    println!();
                }
                Command::PatternDescription { word, colors } => {
                    let pattern = Pattern::<N>::from_description(&word, &colors).unwrap();

                    possible_answers_bk = possible_answers.clone();
                    possible_answers = pattern.filter_words(&possible_answers);

                    if possible_answers.len() == 1 {
                        println!("answer: {}", format!("{}", possible_answers[0]).red());
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn run(&self) {
        while Command::Next == self.game() {}
    }

    fn read_command(&self) -> Command {
        let line = std::io::stdin().lines().next().unwrap().unwrap();
        match line.as_str() {
            ":next" => Command::Next,
            ":exit" => Command::Exit,
            ":clear" => Command::Clear,
            ":show" => Command::Show,
            ":showg" => Command::ShowGuesses,
            ":hard" => Command::Mode(Mode::Hard),
            ":norm" => Command::Mode(Mode::Normal),
            ":undo" => Command::Undo,
            "" | ":guess" => Command::Guess,
            _ => {
                let word = std::iter::once(line)
                    .chain(std::io::stdin().lines().map(Result::unwrap))
                    .find(|word| match word.chars().count() {
                        n if n == N => {
                            if self.valid.contains(&Word::from_str(word).unwrap()) {
                                true
                            } else {
                                println!("no such word in the dictionary");
                                false
                            }
                        }
                        n => {
                            println!("expecting {} letters, found {}", N, n);
                            false
                        }
                    })
                    .unwrap();

                let colors = std::io::stdin()
                    .lines()
                    .map(Result::unwrap)
                    .find(|word| match word.chars().count() {
                        n if n == N => true,
                        n => {
                            println!("expecting {} colors, found {}", N, n);
                            false
                        }
                    })
                    .unwrap();

                Command::PatternDescription { word, colors }
            }
        }
    }

    fn word_list_to_string(words: &[Word<N>]) -> String {
        words.iter().map(|w| format!("{}", w)).join(", ")
    }

    fn show_guesses(sorted_guesses: &[(Word<N>, f64)], words_left: &[Word<N>], show_n: usize) {
        if sorted_guesses.is_empty() {
            println!("couldn't make any guesses");
            return;
        }

        if words_left.is_empty() {
            println!("no possible words left");
            return;
        }

        let threshold = 0.1;

        let mut g1_iter = sorted_guesses
            .iter()
            .filter(|(w, _)| words_left.contains(w));
        let mut g1 = g1_iter.next();

        let mut g2_iter = sorted_guesses
            .iter()
            .filter(|(w, _)| !words_left.contains(w));
        let mut g2 = g2_iter.next();

        let mut xs = vec![];
        let mut last_g2_rank = None;
        let mut i = 0;

        while i < show_n {
            match (g1, g2) {
                (None, None) => break,
                (Some(w1), None) => {
                    i += 1;
                    xs.push((w1, true));
                    g1 = g1_iter.next();
                }
                (None, Some(w2)) => {
                    i += 1;
                    xs.push((w2, false));
                    g2 = g2_iter.next();
                }
                (Some(w1), Some(w2)) => {
                    if w1.1 < w2.1 + threshold {
                        i += 1;
                        xs.push((w1, true));
                        g1 = g1_iter.next();
                    } else if Some(w2.1) == last_g2_rank {
                        g2 = g2_iter.next();
                    } else {
                        i += 1;
                        last_g2_rank = Some(w2.1);
                        xs.push((w2, false));
                        g2 = g2_iter.next();
                    }
                }
            }
        }

        for ((word, rank), in_left) in xs {
            let word_str = if in_left {
                word.to_string().green()
            } else {
                word.to_string().white()
            };

            println!("{word_str}: {rank:.2}");
        }
    }
}
