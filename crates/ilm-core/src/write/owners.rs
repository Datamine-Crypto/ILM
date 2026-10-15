use crate::base::keys::NO_NUMBER;
use crate::write::lessons::WriterLessons;
use crate::write::numbers::{natural_log, share_distance};
use crate::write::reader::TextReader;
use crate::base::first_sight::key_count;
use patterns::{because, source};
use std::cmp::Reverse;

pub struct RunOwners;
source!(RunOwners, "each run of tokens belongs to the slot that its texts are tied to most, measured by the bound of Chernoff on the count of the texts with one group of values; a run that does not hold the word of its slot does not say the slot");

pub const WRITE_RULE_GROUP: usize = 2;
because!(WRITE_RULE_GROUP, RunOwners, "the most values that one rule of the reader names; a group is also at most half of the values of its slot");

pub const WRITE_RULE_CHANCE: f64 = 1e-4;
because!(WRITE_RULE_CHANCE, RunOwners, "the bound on the chance that a run has its count with no tie to the group of its rule");

pub const WRITE_HALF_PARTS: usize = 2;
because!(WRITE_HALF_PARTS, RunOwners, "a group of values of a rule is at most the values of its slot divided by this number");

impl TextReader {
    pub(crate) fn top_values(&self, lessons: &WriterLessons, key: u32, dist: &[(u32, i64)]) -> Vec<(u32, i64)> {
        let limit = (self.key_values[key as usize].rows().len() / WRITE_HALF_PARTS).clamp(1, WRITE_RULE_GROUP);
        let mut seen: Vec<(u32, i64)> = dist.iter().copied().filter(|(value, count)| *value != NO_NUMBER && *count > 0).collect();
        seen.sort_by_key(|(value, count)| (Reverse(*count), lessons.value_rank[*value as usize]));
        seen.truncate(limit);
        seen
    }

    fn tie(&self, lessons: &WriterLessons, key: u32, dist: &[(u32, i64)], texts: usize, total: usize) -> f64 {
        let base = &self.key_values[key as usize];
        let mut best = f64::default();
        let mut held = 0i64;
        let mut all = 0i64;
        for (value, count) in self.top_values(lessons, key, dist) {
            held += count;
            all += key_count(base, &value);
            let share = (all as f64) / (total as f64);
            let got = (held as f64) / (texts as f64);
            if got > share {
                let far = (texts as f64) * share_distance(got, share);
                if far > best {
                    best = far;
                }
            }
        }
        best
    }

    pub(crate) fn own(&mut self, lessons: &WriterLessons, keys: &[u32]) {
        let total = lessons.rows.len();
        let floor = -natural_log(WRITE_RULE_CHANCE);
        self.owner = vec![NO_NUMBER; self.run_keys.len()];
        for run in 0..self.run_keys.len() {
            if self.holds[run].is_empty() {
                continue;
            }
            let mut texts: Vec<u32> = self.holds[run].iter().map(|sentence| self.sent_lesson[*sentence as usize]).collect();
            texts.dedup();
            let units = &self.run_keys[run];
            let mut best: (f64, u32, u32) = (floor, NO_NUMBER, NO_NUMBER);
            let mut with_word = best;
            for key in keys {
                let word = self.word_of[self.key_slot(*key) as usize];
                let values = &self.value_of[*key as usize];
                let mut dist: Vec<(u32, i64)> = Vec::new();
                for lesson in &texts {
                    let value = values[*lesson as usize];
                    match dist.iter_mut().find(|(seen, _)| *seen == value) {
                        Some((_, count)) => *count += 1,
                        None => dist.push((value, 1)),
                    }
                }
                let tie = self.tie(lessons, *key, &dist, texts.len(), total);
                if tie > best.0 {
                    best = (tie, *key, word);
                }
                if word != NO_NUMBER && units.contains(&word) && tie > with_word.0 {
                    with_word = (tie, *key, word);
                }
            }
            if best.2 != NO_NUMBER && !units.contains(&best.2) {
                best = with_word;
            }
            self.owner[run] = best.1;
        }
    }
}
