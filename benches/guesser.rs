use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use rustybovich::{
    guesser_family::{BfsGuesser, BfsGuesserCachedPatterns, BfsGuesserFullCache, Guesser},
    words_family::{Family1, Pattern1, PatternTrait, Word1, WordTrait},
    Dictionary,
};

fn criterion_benchmark(c: &mut Criterion) {
    let Dictionary {
        valid: valid_guesses,
        answers: possible_answers,
    } = Dictionary::<Family1<5>>::read_from_json("assets/en-infinite.json").unwrap();

    // lares -> stalk -> slain
    let answer: Word1<5> = "slain".parse().unwrap();
    let guess_1 = "lares".parse().unwrap();
    let pattern_1 = Pattern1::from_guess(&guess_1, &answer);
    let possible_answers_1 = possible_answers
        .iter()
        .filter(|answer| answer.matches(&pattern_1, &guess_1))
        .copied()
        .collect_vec();

    let guess_2 = "stalk".parse().unwrap();
    let pattern_2 = Pattern1::from_guess(&guess_2, &answer);
    let possible_answers_2 = possible_answers_1
        .iter()
        .filter(|answer| answer.matches(&pattern_2, &guess_2))
        .copied()
        .collect_vec();

    let bfs_guesser = BfsGuesser::<Family1<5>>::new();

    c.bench_function("bfs:slain:lares", |b| {
        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_1, false);
        })
    });

    c.bench_function("bfs:slain:lares-stalk", |b| {
        b.iter(|| {
            let _ = bfs_guesser.rank_guesses(&valid_guesses, &possible_answers_2, false);
        })
    });

    let bfs_cache_guesser = BfsGuesserCachedPatterns::<Family1<5>>::new(&valid_guesses);

    c.bench_function("bfs-cache:slain:lares", |b| {
        b.iter(|| {
            let _ = bfs_cache_guesser.rank_guesses(&valid_guesses, &possible_answers_1, false);
        })
    });

    c.bench_function("bfs-cache:slain:lares-stalk", |b| {
        b.iter(|| {
            let _ = bfs_cache_guesser.rank_guesses(&valid_guesses, &possible_answers_2, false);
        })
    });

    let bfs_full_cache_guesser = BfsGuesserFullCache::<Family1<5>>::new(&valid_guesses);

    c.bench_function("bfs-full-cache:slain:lares", |b| {
        b.iter(|| {
            let _ = bfs_full_cache_guesser.rank_guesses(&valid_guesses, &possible_answers_1, false);
        })
    });

    c.bench_function("bfs-full-cache:slain:lares-stalk", |b| {
        b.iter(|| {
            let _ = bfs_full_cache_guesser.rank_guesses(&valid_guesses, &possible_answers_2, false);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
