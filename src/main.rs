use clap::{builder::PossibleValue, Parser, ValueEnum, ValueHint};
use rustybovich::{
    game::Game,
    guesser::{BfsGuesser, GuesserWrapper, NaiveGuesser},
    Dictionary,
};

#[derive(Clone, Copy, Debug)]
enum GuesserType {
    Naive,
    Bfs,
}

impl ValueEnum for GuesserType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Naive, Self::Bfs]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Naive => PossibleValue::new("naive"),
            Self::Bfs => PossibleValue::new("bfs"),
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
    let dictionary = Dictionary::read_from_json(args.dictionary).unwrap();
    let guesser = match args.guesser {
        GuesserType::Naive => GuesserWrapper::Naive(NaiveGuesser),
        GuesserType::Bfs => GuesserWrapper::Bfs(BfsGuesser),
    };

    let game = Game::<5>::new(dictionary.valid, dictionary.answers, guesser);
    game.run();

    // Dictionary::<5>::read_from_txt("assets/en-nyt-2.valid.txt", "assets/en-nyt-2.answers.txt")
    //     .unwrap()
    //     .write_to_json("assets/en-nyt-2-sorted.json")
    //     .unwrap();

    // Dictionary::<5>::read_from_json("assets/en-nyt.json")
    //     .unwrap()
    //     .write_to_json("assets/en-nyt-sorted.json")
    //     .unwrap();
}
