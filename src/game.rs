use crate::words::{Pattern, Word};
use colored::*;
use itertools::Itertools;
use rayon::prelude::*;

fn expected_left<const N: usize>(guess: &Word<N>, words: &[Word<N>]) -> f64 {
    let matches: usize = words
        .iter()
        .map(|word| {
            let pattern = Pattern::from_guess(guess, word);
            words.iter().filter(|w| w.matches(&pattern)).count()
        })
        .sum();

    (matches as f64) / (words.len() as f64)
}

fn sort_guesses<const N: usize>(guesses: &[Word<N>], words: &[Word<N>]) -> Vec<(Word<N>, f64)> {
    let mut a: Vec<_> = guesses
        .into_par_iter()
        .map(|guess| (*guess, expected_left(guess, words)))
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
    Show,
    ShowGuesses,
    Mode(Mode),
    Undo,
    Guess,
    Pattern(String),
}

pub struct Game<const N: usize> {
    words: Vec<Word<N>>,
}

impl<const N: usize> Game<N> {
    pub fn new(mut words: Vec<String>) -> Self {
        words.sort();

        Self {
            words: words.into_iter().map(Word::from).collect(),
        }
    }

    fn game(&self) -> Command {
        let mut words_left = self.words.clone();
        let mut words_left_bk = self.words.clone();
        let mut guesses = vec![];
        let mut mode = Mode::Normal;

        loop {
            let command = loop {
                println!(
                    "[{mode}] | {} words left | {}:",
                    words_left.len(),
                    "pattern or command".cyan()
                );
                match Self::read_command() {
                    c @ (Command::Next | Command::Exit) => return c,
                    Command::Show => {
                        println!(
                            "{} words left:\n[{}]",
                            words_left.len(),
                            Self::word_list_to_string(&words_left)
                        );
                    }
                    Command::ShowGuesses => {
                        Self::show_guesses(&guesses, &words_left, 10);
                    }
                    Command::Mode(m) => mode = m,
                    Command::Undo => {
                        words_left = words_left_bk.clone();
                    }
                    c @ Command::Guess => break c,
                    Command::Pattern(s) => {
                        if s.len() == N {
                            if self.words.contains(&Word::from(&s)) {
                                break Command::Pattern(s);
                            } else {
                                println!("no such word in the dictionary")
                            }
                        } else {
                            println!("word should be {} letters long", N);
                        }
                    }
                }
            };

            match command {
                Command::Guess => {
                    let guess_from = match mode {
                        Mode::Hard => &words_left,
                        Mode::Normal => &self.words,
                    };
                    guesses = sort_guesses(guess_from, &words_left);

                    Self::show_guesses(&guesses, &words_left, 10);

                    println!();
                }
                Command::Pattern(pattern_word) => {
                    let pattern_desc = loop {
                        let pattern_desk = std::io::stdin().lines().next().unwrap().unwrap();
                        if pattern_desk.len() == N {
                            break pattern_desk;
                        } else {
                            println!("expecting {} colors", N);
                        }
                    };
                    let pattern = Pattern::<N>::from_description(&pattern_word, &pattern_desc);

                    words_left_bk = words_left.clone();
                    words_left = pattern.filter_words(&words_left);

                    if words_left.len() == 1 {
                        println!("answer: {}", format!("{}", words_left[0]).red());
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn run(&self) {
        while Command::Next == self.game() {}
    }

    fn read_command() -> Command {
        let line = std::io::stdin().lines().next().unwrap().unwrap();
        match line.as_str() {
            ":next" => Command::Next,
            ":exit" => Command::Exit,
            ":show" => Command::Show,
            ":showg" => Command::ShowGuesses,
            ":hard" => Command::Mode(Mode::Hard),
            ":norm" => Command::Mode(Mode::Normal),
            ":undo" => Command::Undo,
            "" | ":guess" => Command::Guess,
            _ => Command::Pattern(line),
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

            println!("{word_str}: {rank:.3}");
        }
    }
}
