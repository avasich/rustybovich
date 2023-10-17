use std::{mem::MaybeUninit, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> Word<N> {
    pub fn new(word: [char; N]) -> Self {
        Self { word }
    }

    pub fn matches(&self, pattern: &Pattern<N>) -> bool {
        let mut letters: [_; N] = self.iter().cloned().map(Some).collect_array().unwrap();

        let green_matches = std::iter::zip(letters.iter_mut(), pattern.iter()).all(
            |(letter_option, pattern_letter)| match pattern_letter {
                Colored::Green(letter) => letter_option.take_if(|c| *c == *letter).is_some(),
                _ => true,
            },
        );

        if !green_matches {
            return false;
        }

        pattern.iter().enumerate().all(|(i, c)| match *c {
            Colored::Green(_) => true,
            Colored::Yellow(letter) => letters
                .iter()
                .position(|&other| other.is_some_and(|c| c == letter))
                .and_then(|j| letters[j].take_if(|_| i != j))
                .is_some(),
            Colored::Gray(letter) => letters
                .iter()
                .all(|&other| other.map_or(true, |c| c != letter)),
        })
    }

    // pub fn matches2(&self, pattern: &Pattern<N>) -> bool {
    //     let mut used = [false; N];

    //     let green_matches = pattern.iter().enumerate().all(|(i, c)| match *c {
    //         Colored::Green(c) if self.word[i] == c => {
    //             used[i] = true;
    //             true
    //         }
    //         Colored::Green(_) => false,
    //         _ => true,
    //     });

    //     if !green_matches {
    //         return false;
    //     }

    //     pattern.iter().enumerate().all(|(i, c)| match *c {
    //         Colored::Green(_) => true,
    //         Colored::Yellow(letter) if self.word[i] == letter => false,
    //         Colored::Yellow(letter) => {
    //             match std::iter::zip(used.iter(), self.iter())
    //                 .position(|(&used, &other)| !used && other == letter)
    //             {
    //                 Some(j) => {
    //                     used[j] = true;
    //                     true
    //                 }
    //                 None => false,
    //             }
    //         }
    //         Colored::Gray(letter) => std::iter::zip(used.iter(), self.iter())
    //             .filter_map(|(&used, &other)| (!used).then_some(other))
    //             .all(|other| other != letter),
    //     })
    // }

    fn iter(&self) -> std::slice::Iter<'_, char> {
        self.word.iter()
    }
}

impl<const N: usize> FromStr for Word<N> {
    type Err = IteratorIntoArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word = s.chars().collect_array()?;
        Ok(Self { word })
    }
}

impl<const N: usize> std::fmt::Display for Word<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern<const N: usize> {
    pub pattern: [Colored; N],
}

impl<const N: usize> Pattern<N> {
    pub fn new(pattern: [Colored; N]) -> Self {
        Self { pattern }
    }

    pub fn from_description(word: &str, descr: &str) -> Result<Pattern<N>, IteratorIntoArrayError> {
        let pattern = std::iter::zip(word.chars(), descr.chars())
            .map(|(letter, color)| match color {
                'g' | '1' => letter.green(),
                'y' | '2' => letter.yellow(),
                _ => letter.gray(),
            })
            .collect_array()?;

        Ok(Self { pattern })
    }

    pub fn from_guess(guess: &Word<N>, answer: &Word<N>) -> Self {
        let mut pattern: [_; N] = guess
            .iter()
            .cloned()
            .map(Colored::Gray)
            .collect_array()
            .unwrap();
        let mut word: [_; N] = answer.iter().cloned().map(Some).collect_array().unwrap();

        std::iter::zip(guess.word, answer.word)
            .enumerate()
            .filter(|&(_, (c1, c2))| c1 == c2)
            .for_each(|(i, (c1, _))| {
                pattern[i] = Colored::Green(c1);
                word[i] = None;
            });

        std::iter::zip(guess.word, answer.word)
            .enumerate()
            .filter(|&(_, (c1, c2))| c1 != c2)
            .for_each(|(i, (c1, _))| {
                if let Some(j) = word.iter().position(|&c| c == Some(c1)) {
                    pattern[i] = Colored::Yellow(c1);
                    word[j] = None;
                }
            });

        Self { pattern }
    }

    pub fn match_word(&self, word: &Word<N>) -> bool {
        word.matches(self)
    }

    pub fn filter_words(&self, words: &[Word<N>]) -> Vec<Word<N>> {
        words
            .iter()
            .filter(|word| self.match_word(word))
            .cloned()
            .collect()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Colored> {
        self.pattern.iter()
    }
}

impl<const N: usize> std::fmt::Display for Pattern<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let res = self.iter().fold(String::new(), |mut out, c| {
            let _ = write!(out, "{c}");
            out
        });

        write!(f, "{res}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Colored {
    Green(char),
    Yellow(char),
    Gray(char),
}

impl Colored {
    pub fn value(&self) -> char {
        *match self {
            Colored::Green(c) => c,
            Colored::Yellow(c) => c,
            Colored::Gray(c) => c,
        }
    }
}

impl std::fmt::Display for Colored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        match self {
            Colored::Green(c) => write!(f, "{}", format!("{c}").green()),
            Colored::Yellow(c) => write!(f, "{}", format!("{c}").yellow()),
            Colored::Gray(c) => write!(f, "{}", format!("{c}").white()),
        }
    }
}

trait WrapColored {
    fn green(self) -> Colored;
    fn yellow(self) -> Colored;
    fn gray(self) -> Colored;
}

impl WrapColored for char {
    fn green(self) -> Colored {
        Colored::Green(self)
    }

    fn yellow(self) -> Colored {
        Colored::Yellow(self)
    }

    fn gray(self) -> Colored {
        Colored::Gray(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IteratorIntoArrayError;

trait CollectArray<const N: usize>: std::iter::Iterator {
    fn collect_array(&mut self) -> Result<[Self::Item; N], IteratorIntoArrayError> {
        let mut array: [MaybeUninit<Self::Item>; N] = MaybeUninit::uninit_array();

        for array_ref in array.iter_mut() {
            let value = self.next().ok_or(IteratorIntoArrayError)?;
            array_ref.write(value);
        }

        if self.next().is_none() {
            Ok(unsafe { MaybeUninit::array_assume_init(array) })
        } else {
            Err(IteratorIntoArrayError)
        }
    }
}

impl<const N: usize, I: std::iter::Iterator> CollectArray<N> for I {}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[test]
    fn test_parse_word() {
        assert_eq!(
            Word::from_str("crate"),
            Ok(Word::new(['c', 'r', 'a', 't', 'e']))
        );
        assert_eq!(
            Word::from_str("light"),
            Ok(Word::new(['l', 'i', 'g', 'h', 't']))
        );
        assert_eq!(
            Word::from_str("value"),
            Ok(Word::new(['v', 'a', 'l', 'u', 'e']))
        );
        assert_eq!(Word::<3>::from_str("foo"), Ok(Word::new(['f', 'o', 'o'])))
    }

    #[test]
    fn test_pattern_from_description() {
        assert_eq!(
            Pattern::from_description("crate", "gg..y"),
            Ok(Pattern::new([
                'c'.green(),
                'r'.green(),
                'a'.gray(),
                't'.gray(),
                'e'.yellow(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("moon", "yy.."),
            Ok(Pattern::new([
                'm'.yellow(),
                'o'.yellow(),
                'o'.gray(),
                'n'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("maniac", "y...g."),
            Ok(Pattern::new([
                'm'.yellow(),
                'a'.gray(),
                'n'.gray(),
                'i'.gray(),
                'a'.green(),
                'c'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("gluers", "g.y..."),
            Ok(Pattern::new([
                'g'.green(),
                'l'.gray(),
                'u'.yellow(),
                'e'.gray(),
                'r'.gray(),
                's'.gray(),
            ]))
        );

        assert_eq!(
            Pattern::from_description("bra", "gyy"),
            Ok(Pattern::new(['b'.green(), 'r'.yellow(), 'a'.yellow()]))
        );
    }

    fn word_and_pattern<const N: usize>(
        word: &str,
        pattern_word: &str,
        colors: &str,
    ) -> (Word<N>, Pattern<N>) {
        (
            Word::from_str(word).unwrap(),
            Pattern::from_description(pattern_word, colors).unwrap(),
        )
    }

    trait MatcherBuilder {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Pattern<N>) -> bool;
    }

    struct Matcher1;

    impl MatcherBuilder for Matcher1 {
        fn build<const N: usize>(&self) -> fn(&Word<N>, &Pattern<N>) -> bool {
            Word::matches
        }
    }

    // struct Matcher2;

    // impl MatcherBuilder for Matcher2 {
    //     fn build<const N: usize>(&self) -> fn(&Word<N>, &Pattern<N>) -> bool {
    //         Word::matches2
    //     }
    // }

    fn test_matches_common<B: MatcherBuilder>(matcher_builder: B) {
        let matcher = matcher_builder.build::<3>();

        let (word, pattern) = word_and_pattern("bar", "bra", "gyy");
        assert!(matcher(&word, &pattern));

        let matcher = matcher_builder.build::<5>();

        let (word, pattern) = word_and_pattern("crate", "slate", "..ggg");
        assert!(matcher(&word, &pattern));

        let (word, pattern) = word_and_pattern("abcde", "acbed", "gyyyy");
        assert!(matcher(&word, &pattern));

        let matcher = matcher_builder.build::<6>();

        let (word, pattern) = word_and_pattern("github", "gluers", "g.y...");
        assert!(matcher(&word, &pattern));
    }

    fn test_not_matches_common<B: MatcherBuilder>(matcher_builder: B) {
        let matcher = matcher_builder.build::<3>();

        let (word, pattern) = word_and_pattern("bar", "baz", "gy.");
        assert!(!matcher(&word, &pattern));

        let matcher = matcher_builder.build::<6>();

        let (word, pattern) = word_and_pattern("github", "gluers", "g.y..g");
        assert!(!matcher(&word, &pattern));

        let (word, pattern) = word_and_pattern("github", "gluers", "ggy...");
        assert!(!matcher(&word, &pattern));

        let (word, pattern) = word_and_pattern("github", "gluers", "g.yy..");
        assert!(!matcher(&word, &pattern));
    }

    #[test]
    fn test_matches1() {
        test_matches_common(Matcher1);
    }

    // #[test]
    // fn test_matches2() {
    //     test_not_matches_common(Matcher2);
    // }

    #[test]
    fn test_not_matches1() {
        test_not_matches_common(Matcher1);
    }

    // #[test]
    // fn test_not_matches2() {
    //     test_not_matches_common(Matcher2);
    // }

    fn bench_matches_data_correct_5() -> Vec<(Word<5>, Pattern<5>)> {
        vec![
            word_and_pattern("abcde", "acbed", "gyyyy"),
            word_and_pattern("crate", "slate", "..ggg"),
            word_and_pattern("dicot", "brown", "...y."),
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

    fn bench_matches_data_wrong_5() -> Vec<(Word<5>, Pattern<5>)> {
        vec![
            word_and_pattern("abcde", "acbed", "gyyy."),
            word_and_pattern("crate", "slate", "y.ggg"),
            word_and_pattern("dicot", "brown", ".y.y."),
            word_and_pattern("shirt", "thorp", "y...."),
            word_and_pattern("abcde", "fghij", "....g"),
            word_and_pattern("abcde", "abcde", "yyggg"),
        ]
    }

    // fn bench_matches_data_7() -> Vec<(Word<7>, Pattern<7>)> {
    //     vec![
    //         word_and_pattern("numeral", "lighter", "y....yy"),
    //         word_and_pattern("numeral", "clarets", ".yyyy.."),
    //         word_and_pattern("attests", "steward", "ygy.y.."),
    //         word_and_pattern("attests", "peacock", ".yy...."),
    //     ]
    // }

    #[bench]
    fn bench_matches_correct_size_5_1(b: &mut Bencher) {
        let to_check = bench_matches_data_correct_5();

        b.iter(|| {
            for (word, pattern) in &to_check {
                word.matches(pattern);
            }
        });
    }

    #[bench]
    fn bench_matches_wrong_size_5_1(b: &mut Bencher) {
        let to_check = bench_matches_data_wrong_5();

        b.iter(|| {
            for (word, pattern) in &to_check {
                word.matches(pattern);
            }
        });
    }

    // #[bench]
    // fn bench_matches_correct_size_5_2(b: &mut Bencher) {
    //     let to_check = bench_matches_data_correct_5();

    //     b.iter(|| {
    //         for (word, pattern) in &to_check {
    //             word.matches2(pattern);
    //         }
    //     });
    // }

    // #[bench]
    // fn bench_matches_size_7_1(b: &mut Bencher) {
    //     let to_check = bench_matches_data_7();

    //     b.iter(|| {
    //         for (word, pattern) in &to_check {
    //             word.matches(pattern);
    //         }
    //     });
    // }

    // #[bench]
    // fn bench_matches_size_7_2(b: &mut Bencher) {
    //     let to_check = bench_matches_data_7();

    //     b.iter(|| {
    //         for (word, pattern) in &to_check {
    //             word.matches2(pattern);
    //         }
    //     });
    // }

    // #[bench]
    // fn bench_matches_size_7_3(b: &mut Bencher) {
    //     let to_check = bench_matches_data_7();

    //     b.iter(|| {
    //         for (word, pattern) in &to_check {
    //             word.matches3(pattern);
    //         }
    //     });
    // }
}
