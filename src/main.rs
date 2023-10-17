use rustybovich::game::Game;
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
    let u = serde_json::from_reader(reader)?;
    Ok(u)
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

    let words = read_words_from_file(filename).unwrap();

    let game: Game<5> = Game::new(&words.valid, &words.answers).unwrap();
    game.run();
}
