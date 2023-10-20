use criterion::{criterion_group, criterion_main, Criterion};
use rustybovich::{
    guesser::{BfsGuesser, Guesser, NaiveGuesser},
    words::{Pattern, Word},
    Dictionary,
};

fn criterion_benchmark(c: &mut Criterion) {
    let Dictionary {
        valid: valid_guesses,
        answers: possible_answers,
    } = Dictionary::<5>::from_file("assets/en-infinite.json").unwrap();

    let pattern_cache = Pattern::prepare_all(&valid_guesses, &possible_answers);

    // lares -> stalk -> slain
    let answer: Word<5> = "slain".parse().unwrap();
    let pattern_1 = Pattern::from_guess(&"lares".parse().unwrap(), &answer);
    let possible_answers_1 = pattern_1.filter_words(&possible_answers);

    let pattern_2 = Pattern::from_guess(&"stalk".parse().unwrap(), &answer);
    let possible_answers_2 = pattern_2.filter_words(&possible_answers_1);

    c.bench_function("bfs:slain:lares", |b| {
        let bfs_guesser = BfsGuesser;

        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_1, &pattern_cache);
        })
    });

    c.bench_function("bfs:slain:lares-stalk", |b| {
        let bfs_guesser = BfsGuesser;

        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_2, &pattern_cache);
        })
    });

    c.bench_function("naive:slain:lares", |b| {
        let bfs_guesser = NaiveGuesser;

        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_1, &pattern_cache);
        })
    });

    c.bench_function("naive:slain:lares-stalk", |b| {
        let bfs_guesser = NaiveGuesser;

        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_2, &pattern_cache);
        })
    });
}


criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(30);
    targets = criterion_benchmark
}
criterion_main!(benches);
