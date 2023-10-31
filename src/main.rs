use clap::{builder::PossibleValue, Parser, ValueEnum, ValueHint};
use rustybovich::{
    game::Game,
    guesser_family::{
        bfs_guesser::BfsGuesser,
        bfs_guesser_cached_patterns::BfsGuesserCachedPatterns,
    },
    words_family::words_1::Family1,
    Dictionary,
};

#[derive(Clone, Copy, Debug)]
enum GuesserType {
    Naive,
    Bfs,
    BfsCache,
}

impl ValueEnum for GuesserType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Naive, Self::Bfs, Self::BfsCache]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Naive => PossibleValue::new("naive"),
            Self::Bfs => PossibleValue::new("bfs"),
            Self::BfsCache => PossibleValue::new("bfs-cache"),
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
    let dictionary = Dictionary::<Family1<5>>::read_from_json(args.dictionary).unwrap();
    let guesser = match args.guesser {
        GuesserType::Naive => unreachable!(),
        GuesserType::Bfs => {
            unreachable!();
            // BfsGuesser
        }
        GuesserType::BfsCache => {
            BfsGuesserCachedPatterns::new(&dictionary.valid)
            // unreachable!()
        }
    };

    let game = Game::<Family1<5>, _>::new(
        dictionary.valid.clone(),
        dictionary.answers.clone(),
        guesser,
    );
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
