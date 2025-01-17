#![allow(clippy::type_complexity)]
#![allow(clippy::blocks_in_if_conditions)]

use std::{borrow::Cow, collections::HashSet, num::NonZeroU16};
mod solver;
pub use solver::{Rank, Solver};

// change to 5 or 6
pub const WORD_LENGTH: usize = 5;
pub const GAMES: &str = include_str!("../answers-5.txt");

// static FIRST_GUESS: &str = "presev";
static FIRST_GUESS: &str = "tares";

include!(concat!(env!("OUT_DIR"), "/dictionary.rs"));

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Default for Wordle {
    fn default() -> Self {
        Self::new()
    }
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.iter().copied().map(|(word, _)| word)),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        // Wordle only allows six guesses.
        // We allow more to avoid chopping off the score distribution for stats purposes.
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            if guess == answer {
                guesser.finish(i);
                return Some(i);
            }
            assert!(
                self.dictionary.contains(&*guess),
                "guess '{}' is not in the dictionary",
                guess
            );
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow
    Misplaced,
    /// Gray
    Wrong,
}

impl Correctness {
    fn is_misplaced(letter: u16, answer: &str, used: &mut [bool; WORD_LENGTH]) -> bool {
        let mut enumerated = answer.bytes().enumerate();
        enumerated.any(|(i, a)| {
            if u16::from(a) == letter && !used[i] {
                used[i] = true;
                return true;
            }
            false
        })
    }

    pub fn compute(answer: &str, guess: &str) -> [Self; WORD_LENGTH] {
        assert_eq!(answer.len(), WORD_LENGTH);
        assert_eq!(guess.len(), WORD_LENGTH);
        let mut c = [Correctness::Wrong; WORD_LENGTH];
        let answer_bytes = answer.as_bytes();
        let guess_bytes = guess.as_bytes();
        // Array indexed by lowercase ascii letters
        let mut misplaced = [0u16; (b'z' - b'a' + 1) as usize];

        // Find all correct letters
        for ((&answer, &guess), c) in answer_bytes.iter().zip(guess_bytes).zip(c.iter_mut()) {
            if answer == guess {
                *c = Correctness::Correct
            } else {
                // If the letter does not match, count it as misplaced
                misplaced[(answer - b'a') as usize] += 1;
            }
        }
        // Check all of the non matching letters if they are misplaced
        for (&guess, c) in guess_bytes.iter().zip(c.iter_mut()) {
            // If the letter was guessed wrong and the same letter was counted as misplaced
            if *c == Correctness::Wrong && misplaced[(guess - b'a') as usize] > 0 {
                *c = Correctness::Misplaced;
                misplaced[(guess - b'a') as usize] -= 1;
            }
        }

        c
    }
}

pub const MAX_MASK_ENUM: usize = 3_usize.pow(WORD_LENGTH as u32);

/// A wrapper type for `[Correctness; WORD_LENGTH]` packed into a single byte with a niche.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
// The NonZeroU16 here lets the compiler know that we're not using the value `0`, and that `0` can
// therefore be used to represent `None` for `Option<PackedCorrectness>`.
struct PackedCorrectness(NonZeroU16);

impl From<[Correctness; WORD_LENGTH]> for PackedCorrectness {
    fn from(c: [Correctness; WORD_LENGTH]) -> Self {
        println!();
        let packed = c.iter().fold(0, |acc: u16, c| {
            let old_acc = acc;
            println!("acc {old_acc:#010b} * 3");
            let multiplied_acc = acc.checked_mul(3_u16);
            let multiplied_acc = match multiplied_acc {
                Some(multiplied_acc) => multiplied_acc,
                None => {
                    panic!();
                }
            };

            let correctness_value = match c {
                Correctness::Correct => 0,
                Correctness::Misplaced => 1,
                Correctness::Wrong => 2,
            };

            let sum = multiplied_acc + correctness_value;
            sum
        });
        Self(NonZeroU16::new(packed + 1).unwrap())
    }
}

impl From<PackedCorrectness> for u16 {
    fn from(this: PackedCorrectness) -> Self {
        this.0.get() - 1
    }
}

pub struct Guess<'a> {
    pub word: Cow<'a, str>,
    pub mask: [Correctness; WORD_LENGTH],
}

impl Guess<'_> {
    pub fn matches(&self, word: &str) -> bool {
        // Check if the guess would be possible to observe when `word` is the correct answer.
        // This is equivalent to
        //     Correctness::compute(word, &self.word) == self.mask
        // without _necessarily_ computing the full mask for the tested word
        assert_eq!(word.len(), WORD_LENGTH);
        assert_eq!(self.word.len(), WORD_LENGTH);
        let mut used = [false; WORD_LENGTH];

        // Check Correct letters
        for (i, (a, g)) in word.bytes().zip(self.word.bytes()).enumerate() {
            if a == g {
                if self.mask[i] != Correctness::Correct {
                    return false;
                }
                used[i] = true;
            } else if self.mask[i] == Correctness::Correct {
                return false;
            }
        }

        // Check Misplaced letters
        for (g, e) in self.word.bytes().zip(self.mask.iter()) {
            if *e == Correctness::Correct {
                continue;
            }
            if Correctness::is_misplaced(g.into(), word, &mut used)
                != (*e == Correctness::Misplaced)
            {
                return false;
            }
        }

        // The rest will be all correctly Wrong letters
        true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
    fn finish(&self, _guesses: usize) {}
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
mod tests {
    mod game {
        use crate::{Guess, Wordle, WORD_LENGTH};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }

        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(WORD_LENGTH));
        }

        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == WORD_LENGTH {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        /* use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![C C C C C]);
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W]);
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W]);
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W]);
        }

        #[test]
        fn dremann_from_chat() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W]);
        }

        #[test]
        fn itsapoque_from_chat() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask![W C W W W]);
        }

        #[test]
        fn ricoello_from_chat() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask![C W C C C]);
        } */
    }
}
