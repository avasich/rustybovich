use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use rustybovich::words::{Pattern1, Pattern2, Word1, Word2};

fn pattern_and_words_w1<const N: usize>(
    pattern: &str,
    guess: &str,
    answer: &str,
) -> (Pattern1<N>, Word1<N>, Word1<N>) {
    (
        Pattern1::from_description(guess, pattern).unwrap(),
        Word1::from_str(guess).unwrap(),
        Word1::from_str(answer).unwrap(),
    )
}

fn pattern_and_words_w2<const N: usize>(
    pattern: &str,
    guess: &str,
    answer: &str,
) -> (Pattern2<N>, Word2<N>, Word2<N>) {
    (
        pattern.parse().unwrap(),
        guess.parse().unwrap(),
        answer.parse().unwrap(),
    )
}

fn data_correct_size_5_raw() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("gyyyy", "acbed", "abcde"),
        ("..ggg", "slate", "crate"),
        ("..y..", "brown", "dicot"),
        ("yg.g.", "thorp", "shirt"),
        (".....", "fghij", "abcde"),
        ("ggggg", "abcde", "abcde"),
        ("gggyy", "abced", "abcde"),
        ("..yy.", "xyzaz", "azcde"),
        ("..yy.", "xyzaz", "azcde"),
        ("y....", "light", "elbow"),
        (".y...", "modus", "elbow"),
        ("yyygg", "below", "elbow"),
    ]
}

fn data_incorrect_size_5_raw() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("gyyy.", "acbed", "abcde"),
        ("y.ggg", "slate", "crate"),
        (".y.y.", "brown", "dicot"),
        ("y....", "thorp", "shirt"),
        ("....g", "fghij", "abcde"),
        ("yyggg", "abcde", "abcde"),
    ]
}

fn data_correct_size_7_raw() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("y....yy", "lighter", "numeral"),
        (".yyyy..", "clarets", "numeral"),
        ("ygy.y..", "steward", "attests"),
        (".yy....", "peacock", "attests"),
    ]
}

fn data_correct_size_5_w1() -> Vec<(Pattern1<5>, Word1<5>, Word1<5>)> {
    data_correct_size_5_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w1(pattern, guess, answer))
        .collect_vec()
}

fn data_correct_size_5_w2() -> Vec<(Pattern2<5>, Word2<5>, Word2<5>)> {
    data_correct_size_5_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w2(pattern, guess, answer))
        .collect_vec()
}

fn data_incorrect_size_5_w1() -> Vec<(Pattern1<5>, Word1<5>, Word1<5>)> {
    data_incorrect_size_5_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w1(pattern, guess, answer))
        .collect_vec()
}

fn data_incorrect_size_5_w2() -> Vec<(Pattern2<5>, Word2<5>, Word2<5>)> {
    data_incorrect_size_5_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w2(pattern, guess, answer))
        .collect_vec()
}

fn data_correct_size_7_w1() -> Vec<(Pattern1<7>, Word1<7>, Word1<7>)> {
    data_correct_size_7_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w1(pattern, guess, answer))
        .collect_vec()
}

fn data_correct_size_7_w2() -> Vec<(Pattern2<7>, Word2<7>, Word2<7>)> {
    data_correct_size_7_raw()
        .into_iter()
        .map(|(pattern, guess, answer)| pattern_and_words_w2(pattern, guess, answer))
        .collect_vec()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("from_guess:size_5-w1", |b| {
        let data = data_correct_size_5_w1();
        b.iter(|| {
            data.iter().for_each(|(_pattern, guess, answer)| {
                black_box(Pattern1::from_guess(guess, answer));
            })
        })
    });

    c.bench_function("from_guess:size_5-w2", |b| {
        let data = data_correct_size_5_w2();
        b.iter(|| {
            data.iter().for_each(|(_pattern, guess, answer)| {
                black_box(Pattern2::from_guess(guess, answer));
            })
        })
    });

    c.bench_function("matches_correct:size_5-w1", |b| {
        let data = data_correct_size_5_w1();
        b.iter(|| {
            data.iter().for_each(|(pattern, _guess, answer)| {
                answer.matches(pattern);
            })
        })
    });

    c.bench_function("matches_correct:size_5-w2", |b| {
        let data = data_correct_size_5_w2();
        b.iter(|| {
            data.iter().for_each(|(pattern, guess, answer)| {
                answer.matches(pattern, guess);
            })
        })
    });

    c.bench_function("matches_incorrect:size_5-w1", |b| {
        let data = data_incorrect_size_5_w1();
        b.iter(|| {
            data.iter().for_each(|(pattern, _guess, answer)| {
                answer.matches(pattern);
            })
        })
    });

    c.bench_function("matches_incorrect:size_5-w2", |b| {
        let data = data_incorrect_size_5_w2();
        b.iter(|| {
            data.iter().for_each(|(pattern, guess, answer)| {
                answer.matches(pattern, guess);
            })
        })
    });

    c.bench_function("matches_correct:size_7-w1", |b| {
        let data = data_correct_size_7_w1();
        b.iter(|| {
            data.iter().for_each(|(pattern, _guess, answer)| {
                answer.matches(pattern);
            })
        })
    });

    c.bench_function("matches_correct:size_7-w1", |b| {
        let data = data_correct_size_7_w2();
        b.iter(|| {
            data.iter().for_each(|(pattern, guess, answer)| {
                answer.matches(pattern, guess);
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
