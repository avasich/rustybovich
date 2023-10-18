use std::str::FromStr;

use crate::{
    guesser::Guesser,
    words::{IteratorIntoArrayError, Pattern, Word},
};
use colored::*;
use itertools::Itertools;

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

pub struct Game<const N: usize, G: Guesser<N>> {
    valid: Vec<Word<N>>,
    answers: Vec<Word<N>>,
    guesser: G,
}

impl<const N: usize, G: Guesser<N>> Game<N, G> {
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

        Ok(Self {
            valid,
            answers,
            guesser: G::default(),
        })
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
                    guesses = self.guesser.rank_guesses(valid_guesses, &possible_answers);
                    Self::show_guesses(&guesses, &possible_answers, 10);

                    println!();
                }
                Command::PatternDescription { word, colors } => {
                    let pattern = Pattern::from_description(&word, &colors).unwrap();

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

    fn show_guesses(sorted_guesses: &[(Word<N>, f32)], words_left: &[Word<N>], show_n: usize) {
        if sorted_guesses.is_empty() {
            println!("couldn't make any guesses");
            return;
        }

        if words_left.is_empty() {
            println!("no possible words left");
            return;
        }

        let threshold = 0.1;

        let mut prev_rank = 0;
        let mut n = 0;

        sorted_guesses
            .iter()
            .map(|(word, rank)| (word, rank, words_left.contains(word)))
            .coalesce(|prev, curr| {
                if curr.1 - prev.1 < threshold {
                    return match (prev.2, curr.2) {
                        (true, true) => Err((prev, curr)),
                        (true, false) => Ok(prev),
                        (false, true) => Ok(curr),
                        (false, false) => Ok(prev),
                    };
                }
                Err((prev, curr))
            })
            .take_while(|(_, rank, _)| {
                n += 1;
                let rank_int = (**rank * 100.0).round() as i32;
                if n < show_n {
                    prev_rank = rank_int;
                    true
                } else {
                    prev_rank == rank_int
                }
            })
            .for_each(|(word, rank, left)| {
                let word_str = if left {
                    word.to_string().green()
                } else {
                    word.to_string().white()
                };

                println!("{word_str}: {rank:.2}");
            });
    }
}
