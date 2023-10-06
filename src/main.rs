use std::{error::Error, fs::File, io::BufReader, path::Path};

use colored::*;
use itertools::Itertools;
use rayon::prelude::*;

use rustybovich::{Pattern, Word};

fn read_words_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

enum Command {
    Next,
    Exit,
    Show,
    Mode(Mode),
    Value(String),
}

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

fn read_command() -> Command {
    let line = std::io::stdin().lines().next().unwrap().unwrap();
    match line.as_str() {
        "!next" => Command::Next,
        "!exit" => Command::Exit,
        "!show" => Command::Show,
        "!hard" => Command::Mode(Mode::Hard),
        "!norm" => Command::Mode(Mode::Normal),
        _ => Command::Value(line),
    }
}

fn word_list_to_string<const N: usize>(words: &[Word<N>]) -> String {
    words.iter().map(|w| format!("{}", w)).join(", ")
}

fn game<const N: usize>(words: Vec<Word<N>>, mut mode: Mode) -> Command {
    let mut words_left = words.clone();
    loop {
        let pattern_word = loop {
            println!("[{mode}] {}:", "pattern or command".cyan());
            match read_command() {
                c @ (Command::Next | Command::Exit) => return c,
                Command::Show => println!(
                    "{} words left:\n[{}]",
                    words_left.len(),
                    word_list_to_string(&words_left)
                ),
                Command::Mode(m) => mode = m,
                Command::Value(s) => break s,
            }
        };
        let pattern_desc = std::io::stdin().lines().next().unwrap().unwrap();
        let pattern = Pattern::<N>::from_description(&pattern_word, &pattern_desc);

        words_left = pattern.filter_words(&words_left);

        println!("{pattern}");
        println!("{}", format!("{} words left", words_left.len()).cyan());

        loop {
            println!("[{mode}] | {}", "command or empty line to continue".cyan());

            match read_command() {
                c @ (Command::Next | Command::Exit) => return c,
                Command::Show => println!("[{}]", word_list_to_string(&words_left)),
                Command::Mode(m) => mode = m,
                Command::Value(_) => break,
            }

            println!();
        }

        println!("computing guesses...");

        let guesses = match mode {
            Mode::Hard => &words_left,
            Mode::Normal => &words,
        };

        let guesses = top_guesses(guesses, &words_left, 10);

        for (guess, n) in guesses {
            println!("{guess}: {n}");
        }

        println!();
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let filename = args
        .get(1)
        .and_then(|x| x.split_once('='))
        .and_then(|x| match x {
            ("--file", filename) => Some(filename),
            _ => None,
        })
        .unwrap_or("assets/en-nyt.json");

    let words: Vec<Word<5>> = read_words_from_file(filename)
        .expect("failed to load file")
        .into_iter()
        .map(Word::from)
        .collect();

    loop {
        use colored::*;
        println!("{}", "NEW GAME".red());
        match game(words.clone(), Mode::Normal) {
            Command::Next => continue,
            Command::Exit => return,
            _ => unreachable!(),
        }
    }
}

fn expected_left<const N: usize>(guess: &Word<N>, words: &[Word<N>]) -> u64 {
    let matches: usize = words
        .iter()
        .map(|word| {
            let pattern = Pattern::from_guess(guess, word);
            words.iter().filter(|w| w.matches(&pattern)).count()
        })
        .sum();

    (matches as u64) / (words.len() as u64)
}

fn top_guesses<const N: usize>(
    guesses: &[Word<N>],
    words: &[Word<N>],
    top_n: usize,
) -> Vec<(Word<N>, u64)> {
    let mut a: Vec<_> = guesses
        .into_par_iter()
        .map(|guess| (*guess, expected_left(guess, words)))
        .collect();

    a.as_parallel_slice_mut().sort_unstable_by_key(|(_w, n)| *n);

    a.into_iter().take(top_n).collect()
}
