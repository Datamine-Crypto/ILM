use crate::base::keys::KeyNumbers;
use patterns::{because, source};

pub struct RunsAsNumbers;
source!(RunsAsNumbers, "runs of numbers as numbers: each different run of word numbers, of any length, gets the next whole number once, so a table over phrases is a flat array. A run is looked up by a wide hash of its numbers, and the run that a place holds is compared with the run that is asked for, so two runs with one hash are still two runs");

pub struct WordFold;
source!(WordFold, "the hash of Fowler, Noll and Vo taken over whole numbers in place of bytes: each number is mixed in and the sum is multiplied by a fixed odd number; two such folds with different factors give the two halves of a wide hash");

const LOW_FOLD: u64 = 0x9e37_79b9_7f4a_7c15;
because!(LOW_FOLD, WordFold, "the factor of the fold that gives the low half of the hash: an odd number with no short pattern of bits, so each number of a run changes all higher bits");

const HIGH_FOLD: u64 = 0xc2b2_ae3d_27d4_eb4f;
because!(HIGH_FOLD, WordFold, "the factor of the fold that gives the high half of the hash: another odd number with no short pattern of bits, so the two halves do not fail together");

const HALF_TURN: u32 = 31;
because!(HALF_TURN, WordFold, "the bits by which the high fold is turned after each number, so that it does not follow the low fold step by step");

const FOLD_WIDTH: u32 = 64;
because!(FOLD_WIDTH, WordFold, "the width of each of the two folds; together they fill the wide key of the table");

fn run_key(run: &[u32], salt: u64) -> u128 {
    let mut low = salt ^ (run.len() as u64);
    let mut high = !salt;
    for word in run {
        low = (low ^ u64::from(*word)).wrapping_mul(LOW_FOLD);
        high = (high ^ u64::from(*word)).wrapping_mul(HIGH_FOLD).rotate_left(HALF_TURN);
    }
    u128::from(low) | (u128::from(high) << FOLD_WIDTH)
}
because!(run_key, WordFold, "the wide hash of a run of numbers; the salt gives another hash for the same run, for the case that two runs have one hash");

#[derive(Clone, Debug, Default)]
pub struct RunNumbers {
    keys: KeyNumbers,
    words: Vec<u32>,
    ends: Vec<u32>,
}
because!(RunNumbers, RunsAsNumbers, WordFold, "the runs that were seen, each with its number, which is its place in the order of first sight. number gives the number of a run and a new number to a new run; find gives the number of a run that was seen and nothing for a new one, and changes nothing, so many threads read one table; run gives the numbers of a run back; count says how many runs the table holds");

impl RunNumbers {
    pub fn count(&self) -> usize {
        self.keys.keys().len()
    }

    pub fn run(&self, number: u32) -> &[u32] {
        let from = number.checked_sub(1).map_or(0, |before| self.ends[before as usize] as usize);
        &self.words[from..self.ends[number as usize] as usize]
    }

    pub fn find(&self, run: &[u32]) -> Option<u32> {
        let mut salt = 0;
        loop {
            let number = self.keys.find(run_key(run, salt))?;
            if self.run(number) == run {
                return Some(number);
            }
            salt += 1;
        }
    }

    pub fn number(&mut self, run: &[u32]) -> u32 {
        let mut salt = 0;
        loop {
            let number = self.keys.number(run_key(run, salt));
            if number as usize == self.ends.len() {
                self.words.extend_from_slice(run);
                self.ends.push(self.words.len() as u32);
                return number;
            }
            if self.run(number) == run {
                return number;
            }
            salt += 1;
        }
    }
}

pub fn run_place(tokens: &[u32], run: &[u32], from: usize) -> Option<usize> {
    if run.is_empty() || tokens.len() < run.len() {
        return None;
    }
    (from..=tokens.len() - run.len()).find(|at| tokens[*at..*at + run.len()] == *run)
}
because!(run_place, RunsAsNumbers, "the first place at or after a start where a run of word numbers stands in a sentence, and nothing for an empty run or one longer than the sentence");
