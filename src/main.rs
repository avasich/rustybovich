use clap::{builder::PossibleValue, Parser, ValueEnum, ValueHint};
use rustybovich::{
    game::Game,
    guesser_family::{
        bfs_guesser::BfsGuesser,
        bfs_guesser_cached_patterns::BfsGuesserCachedPatterns,
        BfsGuesserFullCache,
        Guesser,
    },
    words_family::words_1::Family1,
    Dictionary,
};

#[derive(Clone, Copy, Debug)]
enum GuesserType {
    Naive,
    Bfs,
    BfsCache,
    BfsFullCache,
}

impl ValueEnum for GuesserType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Naive, Self::Bfs, Self::BfsCache, Self::BfsFullCache]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Naive => PossibleValue::new("naive"),
            Self::Bfs => PossibleValue::new("bfs"),
            Self::BfsCache => PossibleValue::new("bfs-cache"),
            Self::BfsFullCache => PossibleValue::new("bfs-full-cache"),
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
    let guesser: Box<dyn Guesser<Family1<5>>> = match args.guesser {
        GuesserType::Naive => unreachable!(),
        GuesserType::Bfs => Box::new(BfsGuesser::new()),
        GuesserType::BfsCache => Box::new(BfsGuesserCachedPatterns::new(&dictionary.valid)),
        GuesserType::BfsFullCache => Box::new(BfsGuesserFullCache::new(&dictionary.valid)),
    };

    let game = Game::<Family1<5>>::new(&dictionary.valid, &dictionary.answers, guesser);
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
