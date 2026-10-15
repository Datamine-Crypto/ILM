use crate::base::keys::NO_NUMBER;
use crate::write::lessons::WriterLessons;
use crate::write::numbers::{natural_log, share_distance};
use crate::write::owners::{WRITE_RULE_CHANCE, WRITE_RULE_GROUP};
use crate::write::reader::{ReadRule, TextReader, WRITE_RULE_TEXTS, run_units};
use crate::base::first_sight::{NumberBag, NumberTable};
use patterns::{because, source};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct CoveringRules;
source!(CoveringRules, "learning a list of rules by covering, as Michalski and as Clark and Niblett describe it: the sure rule with the most texts is taken, the sentences that it reads are taken away, and the rest is counted again; inside the sentences of a rule the same count gives its finer rules");

pub const WRITE_RULE_SURE: f64 = 0.9;
because!(WRITE_RULE_SURE, CoveringRules, "the share of the lesson texts with the run of a rule whose facts hold a value of its group");

pub const WRITE_RULE_DEPTH: usize = 2;
because!(WRITE_RULE_DEPTH, CoveringRules, "how deep the rules inside the sentences of a rule go");

fn group_superset(group: &[u32; WRITE_RULE_GROUP], parent: &[u32; WRITE_RULE_GROUP]) -> bool {
    parent.iter().all(|value| *value == NO_NUMBER || group.contains(value))
}
because!(group_superset, CoveringRules, "does a group of values hold every value of the group of the rule above it; such a finer rule would say nothing new");

fn sorted_group(values: &[u32]) -> [u32; WRITE_RULE_GROUP] {
    let mut group = [NO_NUMBER; WRITE_RULE_GROUP];
    group[..values.len()].copy_from_slice(values);
    group.sort();
    group
}
because!(sorted_group, CoveringRules, "a group of values in one fixed order, so that two groups with the same values are the same group");

impl TextReader {
    fn sure(&self, lessons: &WriterLessons, key: u32, dist: &[(u32, i64)], texts: i64, base: &NumberTable<u32, i64>, total: usize, parent: Option<&[u32; WRITE_RULE_GROUP]>) -> Option<([u32; WRITE_RULE_GROUP], i64)> {
        let mut group: Vec<u32> = Vec::new();
        let mut held = 0i64;
        for (value, count) in self.top_values(lessons, key, dist) {
            group.push(value);
            held += count;
            if (held as f64) < WRITE_RULE_SURE * (texts as f64) {
                continue;
            }
            let found = sorted_group(&group);
            if parent.is_some_and(|above| group_superset(&found, above)) {
                return None;
            }
            let all: i64 = group.iter().map(|value| base.get(value).copied().unwrap_or(0)).sum();
            let share = (all as f64) / (total as f64);
            let got = (held as f64) / (texts as f64);
            if got <= share || (texts as f64) * share_distance(got, share) < -natural_log(WRITE_RULE_CHANCE) {
                return None;
            }
            return Some((found, held));
        }
        None
    }

    fn cover(&self, lessons: &WriterLessons, key: u32, ids: &[u32], parent: Option<&[u32; WRITE_RULE_GROUP]>, depth: usize) -> Vec<ReadRule> {
        let values = &self.value_of[key as usize];
        let mut alive: NumberBag<u32> = ids.iter().copied().collect();
        let mut times: NumberTable<(u32, u32), i64> = NumberTable::default();
        let mut texts: NumberBag<u32> = NumberBag::default();
        for sentence in ids {
            let lesson = self.sent_lesson[*sentence as usize];
            texts.insert(lesson);
            for run in &self.per[*sentence as usize] {
                if self.owner[*run as usize] == key {
                    *times.entry((lesson, *run)).or_insert(0) += 1;
                }
            }
        }
        let mut base: NumberTable<u32, i64> = NumberTable::default();
        for lesson in &texts {
            *base.entry(values[*lesson as usize]).or_insert(0) += 1;
        }
        let mut count: NumberTable<u32, i64> = NumberTable::default();
        let mut dist: NumberTable<u32, Vec<(u32, i64)>> = NumberTable::default();
        for (lesson, run) in times.keys() {
            *count.entry(*run).or_insert(0) += 1;
            let seen = dist.entry(*run).or_default();
            let value = values[*lesson as usize];
            match seen.iter_mut().find(|(one, _)| *one == value) {
                Some((_, many)) => *many += 1,
                None => seen.push((value, 1)),
            }
        }
        let entry = |run: u32, many: i64| (many, Reverse(run_units(&self.run_keys[run as usize])), Reverse(self.run_order[run as usize]), run);
        let mut heap: BinaryHeap<(i64, Reverse<usize>, Reverse<u32>, u32)> = count.iter().filter(|(_, many)| **many >= WRITE_RULE_TEXTS).map(|(run, many)| entry(*run, *many)).collect();
        let mut rules: Vec<ReadRule> = Vec::new();
        loop {
            let mut best: Option<((i64, i64, i64), u32, [u32; WRITE_RULE_GROUP])> = None;
            let mut others: Vec<u32> = Vec::new();
            while let Some(top) = heap.peek() {
                if best.as_ref().is_some_and(|(rank, _, _)| top.0 < rank.0) {
                    break;
                }
                let (many, _, _, run) = heap.pop().unwrap_or_default();
                let now = count.get(&run).copied().unwrap_or(0);
                if many != now {
                    continue;
                }
                let Some((group, held)) = self.sure(lessons, key, &dist[&run], now, &base, texts.len(), parent) else {
                    continue;
                };
                let rank = (held, -now, -(run_units(&self.run_keys[run as usize]) as i64));
                match &best {
                    Some((top_rank, top_run, _)) if rank <= *top_rank => {
                        let _ = top_run;
                        others.push(run);
                    }
                    _ => {
                        if let Some((_, top_run, _)) = &best {
                            others.push(*top_run);
                        }
                        best = Some((rank, run, group));
                    }
                }
            }
            for run in others {
                heap.push(entry(run, count.get(&run).copied().unwrap_or(0)));
            }
            let Some(((held, _, _), run, group)) = best else {
                break;
            };
            let had = count.get(&run).copied().unwrap_or(0);
            let covered: Vec<u32> = self.holds[run as usize].iter().copied().filter(|sentence| alive.contains(sentence)).collect();
            let mut changed: Vec<u32> = Vec::new();
            for sentence in &covered {
                alive.remove(sentence);
                let lesson = self.sent_lesson[*sentence as usize];
                for other in &self.per[*sentence as usize] {
                    if self.owner[*other as usize] != key {
                        continue;
                    }
                    let left = times.entry((lesson, *other)).or_insert(0);
                    *left -= 1;
                    if *left == 0 {
                        *count.entry(*other).or_insert(0) -= 1;
                        let value = values[lesson as usize];
                        if let Some((_, many)) = dist.entry(*other).or_default().iter_mut().find(|(one, _)| *one == value) {
                            *many -= 1;
                        }
                        changed.push(*other);
                    }
                }
            }
            changed.sort();
            changed.dedup();
            for other in changed {
                let now = count.get(&other).copied().unwrap_or(0);
                if now >= WRITE_RULE_TEXTS {
                    heap.push(entry(other, now));
                }
            }
            let finer = if depth < WRITE_RULE_DEPTH { self.cover(lessons, key, &covered, Some(&group), depth + 1) } else { Vec::new() };
            rules.push(ReadRule { run, group, had, held, finer });
        }
        rules
    }

    pub(crate) fn set_top(&mut self) {
        self.top = vec![(NO_NUMBER, NO_NUMBER); self.run_keys.len()];
        for (key, rules) in self.rules.iter().enumerate() {
            if !self.read_keys[key] {
                continue;
            }
            for (place, rule) in rules.iter().enumerate() {
                self.top[rule.run as usize] = (key as u32, place as u32);
            }
        }
    }

    pub(crate) fn own_and_cover(&mut self, lessons: &WriterLessons, keys: &[u32]) {
        self.own(lessons, keys);
        let all: Vec<u32> = (0..self.sent_toks.len() as u32).collect();
        self.rules = vec![Vec::new(); self.keys.len()];
        self.read_keys = vec![false; self.keys.len()];
        for key in keys {
            self.rules[*key as usize] = self.cover(lessons, *key, &all, None, 0);
            self.read_keys[*key as usize] = true;
        }
        self.set_top();
    }
}
