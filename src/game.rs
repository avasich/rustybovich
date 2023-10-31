use std::str::FromStr;

use colored::*;
use itertools::Itertools;

use crate::{
    guesser_family::Guesser,
    words_family::{Family, WordTrait},
};

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
    PatternDescription { colors: String, word: String },
}

pub struct Game<F: Family, G: Guesser<F>> {
    valid: Vec<F::Word>,
    answers: Vec<F::Word>,
    guesser: G,
}

impl<F: Family, G: Guesser<F>> Game<F, G>
where
    <F::Word as FromStr>::Err: std::fmt::Debug,
    <F::Pattern as FromStr>::Err: std::fmt::Debug,
{
    const N: usize = 5;
    
    pub fn new(valid: Vec<F::Word>, answers: Vec<F::Word>, guesser: G) -> Self {
        Self {
            valid,
            answers,
            guesser,
        }
    }

    fn game(&self) -> Command {
        let mut possible_answers = self.answers.clone();
        let mut possible_answers_bk = possible_answers.clone();
        let mut ranked_guesses = vec![];
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
                    Command::ShowGuesses => {
                        Self::show_guesses(&ranked_guesses, &possible_answers, 10)
                    }
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
                    ranked_guesses = self.guesser.rank_guesses(
                        valid_guesses,
                        &possible_answers,
                        // pattern_cache
                    );

                    {
                        println!("writing guesses");
                        use std::fs::{self};
                        fs::write(
                            "tmp/ranked_dump.txt",
                            ranked_guesses
                                .iter()
                                .map(|(w, r)| format!("{w} : {r}"))
                                .join("\n"),
                        )
                        .expect("failed to write answers");
                    }

                    Self::show_guesses(&ranked_guesses, &possible_answers, 10);

                    println!();
                }
                Command::PatternDescription { colors, word } => {
                    // let pattern = Pattern1::from_description(&colors, &word).unwrap();
                    let pattern = F::Pattern::from_str(&colors).unwrap();
                    let guess = word.parse().unwrap();

                    possible_answers_bk.clone_from(&possible_answers);
                    possible_answers = possible_answers
                        .into_iter()
                        .filter(|answer| answer.matches(&pattern, &guess))
                        .collect_vec();
                    // possible_answers = pattern.filter_words(&possible_answers);

                    // {
                    //     use std::fs::{self};
                    //     fs::write(
                    //         "tmp/answers_dump.txt",
                    //         possible_answers.iter().map(Word1::to_string).join("\n"),
                    //     )
                    //     .expect("failed to write answers");
                    // }

                    if possible_answers.len() == 1 {
                        println!("answer: {}", format!("{}", possible_answers[0]).red());
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn run(&self) {
        // let pattern_cache = Pattern1::prepare_all(&self.valid, &self.answers);
        while Command::Next
            == self.game(
            // &pattern_cache
        ) {}
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
                        n if n == Self::N => {
                            if self.valid.contains(&F::Word::from_str(word).unwrap()) {
                                true
                            } else {
                                println!("no such word in the dictionary");
                                false
                            }
                        }
                        n => {
                            println!("expecting {} letters, found {}", Self::N, n);
                            false
                        }
                    })
                    .unwrap();

                let colors = std::io::stdin()
                    .lines()
                    .map(Result::unwrap)
                    .find(|word| match word.chars().count() {
                        n if n == Self::N => true,
                        n => {
                            println!("expecting {} colors, found {}", Self::N, n);
                            false
                        }
                    })
                    .unwrap();

                Command::PatternDescription { word, colors }
            }
        }
    }

    fn word_list_to_string(words: &[F::Word]) -> String {
        words.iter().map(|w| format!("{}", w)).join(", ")
    }

    fn show_guesses(ranked_guesses: &[(F::Word, f32)], answers_left: &[F::Word], show_n: usize) {
        if ranked_guesses.is_empty() {
            println!("couldn't make any guesses");
            return;
        }

        if answers_left.is_empty() {
            println!("no possible words left");
            return;
        }

        let threshold = 0.1;

        let mut prev_rank = 0;
        let mut n = 0;

        ranked_guesses
            .iter()
            .map(|(word, rank)| (word, rank, answers_left.contains(word)))
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
