use clap::{builder::PossibleValue, Parser, ValueEnum, ValueHint};
use rustybovich::{
    game::Game,
    guesser::{BFSBruteforceGuesser, BFSSmartGuesser, GuesserWrapper, NaiveGuesser},
};
use serde::Deserialize;
use std::{error::Error, fs::File, io::BufReader, path::Path};

#[derive(Deserialize)]
struct WordsFile {
    valid: Vec<String>,
    answers: Vec<String>,
}

fn read_words_from_file<P: AsRef<Path>>(path: P) -> Result<WordsFile, Box<dyn Error>> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[derive(Clone, Copy, Debug)]
enum GuesserType {
    Naive,
    BFSSmart,
    BFSStupid,
}

impl ValueEnum for GuesserType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Naive, Self::BFSSmart, Self::BFSStupid]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Naive => PossibleValue::new("naive"),
            Self::BFSSmart => PossibleValue::new("bfs_smart"),
            Self::BFSStupid => PossibleValue::new("bfs_stupid"),
        })
    }
}

#[derive(Parser, Debug)]
struct Arguments {
    #[arg(short = 'g', default_value = "naive")]
    guesser: GuesserType,

    #[arg(short = 'd', value_name = "FILE", value_hint = ValueHint::FilePath)]
    dictionary: std::path::PathBuf,
}

fn main() {
    let args = Arguments::parse();

    let WordsFile { valid, answers } = read_words_from_file(args.dictionary).unwrap();
    let guesser = match args.guesser {
        GuesserType::Naive => GuesserWrapper::Naive(NaiveGuesser),
        GuesserType::BFSSmart => GuesserWrapper::BFSSmart(BFSSmartGuesser),
        GuesserType::BFSStupid => GuesserWrapper::BFSStupid(BFSBruteforceGuesser),
    };

    let game = Game::<5>::new(&valid, &answers, guesser).unwrap();
    game.run();
}
