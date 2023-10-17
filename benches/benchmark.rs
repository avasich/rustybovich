use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustybovich::words::{Pattern, Word};

fn word_and_pattern<const N: usize>(
    word: &str,
    pattern_word: &str,
    colors: &str,
) -> (Word<N>, Word<N>, Pattern<N>) {
    (
        Word::from_str(word).unwrap(),
        Word::from_str(pattern_word).unwrap(),
        Pattern::from_description(pattern_word, colors).unwrap(),
    )
}

fn data_words_and_patterns_correct_size_5() -> Vec<(Word<5>, Word<5>, Pattern<5>)> {
    vec![
        word_and_pattern("abcde", "acbed", "gyyyy"),
        word_and_pattern("crate", "slate", "..ggg"),
        word_and_pattern("dicot", "brown", "..y.."),
        word_and_pattern("shirt", "thorp", "yg.g."),
        word_and_pattern("abcde", "fghij", "....."),
        word_and_pattern("abcde", "abcde", "ggggg"),
        word_and_pattern("abcde", "abced", "gggyy"),
        word_and_pattern("azcde", "xyzaz", "..yy."),
        word_and_pattern("azcde", "xyzaz", "..yy."),
        word_and_pattern("elbow", "light", "y...."),
        word_and_pattern("elbow", "modus", ".y..."),
        word_and_pattern("elbow", "below", "yyygg"),
    ]
}

fn data_words_and_patterns_incorrect_size_5() -> Vec<(Word<5>, Word<5>, Pattern<5>)> {
    vec![
        word_and_pattern("abcde", "acbed", "gyyy."),
        word_and_pattern("crate", "slate", "y.ggg"),
        word_and_pattern("dicot", "brown", ".y.y."),
        word_and_pattern("shirt", "thorp", "y...."),
        word_and_pattern("abcde", "fghij", "....g"),
        word_and_pattern("abcde", "abcde", "yyggg"),
    ]
}

fn data_words_and_patterns_correct_size_7() -> Vec<(Word<7>, Word<7>, Pattern<7>)> {
    vec![
        word_and_pattern("numeral", "lighter", "y....yy"),
        word_and_pattern("numeral", "clarets", ".yyyy.."),
        word_and_pattern("attests", "steward", "ygy.y.."),
        word_and_pattern("attests", "peacock", ".yy...."),
    ]
}

fn criterion_benchmark(c: &mut Criterion) {
    let correct_size_5 = data_words_and_patterns_correct_size_5();

    c.bench_function("from_guess_1", |b| {
        b.iter(|| {
            for (answer, guess, _) in &correct_size_5 {
                black_box(Pattern::from_guess(guess, answer));
            }
        })
    });

    c.bench_function("from_guess_2", |b| {
        b.iter(|| {
            for (answer, guess, _) in &correct_size_5 {
                black_box(Pattern::from_guess2(guess, answer));
            }
        })
    });

    c.bench_function("matches_correct_size_5_1", |b| {
        b.iter(|| {
            for (word, _, pattern) in &correct_size_5 {
                word.matches(pattern);
            }
        })
    });

    let incorrect_size_5 = data_words_and_patterns_incorrect_size_5();

    c.bench_function("matches_incorrect_size_5_1", |b| {
        b.iter(|| {
            for (word, _, pattern) in &incorrect_size_5 {
                word.matches(pattern);
            }
        })
    });

    let correct_size_7 = data_words_and_patterns_correct_size_7();

    c.bench_function("matches_correct_size_7_1", |b| {
        b.iter(|| {
            for (word, _, pattern) in &correct_size_7 {
                word.matches(pattern);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
