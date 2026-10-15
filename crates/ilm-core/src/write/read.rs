use crate::base::keys::NO_NUMBER;
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, WRITE_LONGEST_RUN, WriterLessons, slot_bit};
use crate::base::runs::run_place;
use crate::write::owners::WRITE_RULE_GROUP;
use crate::write::reader::{ReadFound, TextReader};
use patterns::source;

pub struct SentenceReading;
source!(SentenceReading, "a sentence is read for each slot by the first rule of the list that stands in it, then by the first of its finer rules; a sentence says a list of facts when every thing that the reader finds in it is a given fact");

impl TextReader {
    pub fn read(&self, toks: &[u32]) -> Vec<ReadFound> {
        let mut places: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
        for size in 1..=WRITE_LONGEST_RUN {
            if toks.len() < size {
                break;
            }
            for at in 0..=toks.len() - size {
                let Some(run) = self.run_number(&toks[at..at + size]) else {
                    continue;
                };
                if self.owner[run as usize] == NO_NUMBER {
                    continue;
                }
                let span = (at as u32, (at + size) as u32);
                match places.iter_mut().find(|(seen, _)| *seen == run) {
                    Some((_, spans)) => spans.push(span),
                    None => places.push((run, vec![span])),
                }
            }
        }
        let mut first: Vec<(u32, u32, u32)> = Vec::new();
        for (run, _) in &places {
            let (key, place) = self.top[*run as usize];
            if key == NO_NUMBER {
                continue;
            }
            match first.iter_mut().find(|(seen, _, _)| *seen == key) {
                Some(entry) => {
                    if place < entry.1 {
                        *entry = (key, place, *run);
                    }
                }
                None => first.push((key, place, *run)),
            }
        }
        let spans_of = |run: u32| places.iter().find(|(seen, _)| *seen == run).map(|(_, spans)| spans.as_slice());
        let mut out: Vec<ReadFound> = Vec::with_capacity(first.len());
        for (key, place, _) in first {
            let mut rule = &self.rules[key as usize][place as usize];
            let mut spans: Vec<(u32, u32)> = spans_of(rule.run).map(<[(u32, u32)]>::to_vec).unwrap_or_default();
            while let Some(finer) = rule.finer.iter().find(|finer| spans_of(finer.run).is_some()) {
                rule = finer;
                spans.extend_from_slice(spans_of(rule.run).unwrap_or_default());
            }
            out.push(ReadFound { key, group: rule.group, spans });
        }
        out
    }

    pub fn read_groups(&self, toks: &[u32], present: &mut Vec<u32>, out: &mut Vec<(u32, [u32; WRITE_RULE_GROUP])>) {
        present.clear();
        out.clear();
        for size in 1..=WRITE_LONGEST_RUN {
            if toks.len() < size {
                break;
            }
            for at in 0..=toks.len() - size {
                let Some(run) = self.run_number(&toks[at..at + size]) else {
                    continue;
                };
                if self.owner[run as usize] != NO_NUMBER && !present.contains(&run) {
                    present.push(run);
                }
            }
        }
        let mut first: Vec<(u32, u32)> = Vec::new();
        for run in present.iter() {
            let (key, place) = self.top[*run as usize];
            if key == NO_NUMBER {
                continue;
            }
            match first.iter_mut().find(|(seen, _)| *seen == key) {
                Some(entry) => {
                    if place < entry.1 {
                        entry.1 = place;
                    }
                }
                None => first.push((key, place)),
            }
        }
        for (key, place) in first {
            let mut rule = &self.rules[key as usize][place as usize];
            while let Some(finer) = rule.finer.iter().find(|finer| present.contains(&finer.run)) {
                rule = finer;
            }
            out.push((key, rule.group));
        }
    }

    pub fn other_name(&self, lessons: &WriterLessons, toks: &[u32]) -> bool {
        toks.iter().enumerate().any(|(at, token)| lessons.sure_names.get(token).is_some_and(|names| names.iter().any(|name| toks[at..].starts_with(name))))
    }

    pub fn said(&self, lessons: &WriterLessons, given: &GivenFacts, value_toks: &[(u32, Vec<u32>)], toks: &[u32], placed: &[(u32, Vec<u32>)]) -> Option<u64> {
        if self.other_name(lessons, toks) {
            return None;
        }
        let mut out = 0u64;
        let mut items: Vec<(u32, Vec<u32>)> = placed.to_vec();
        for fact in &given.facts {
            if !lessons.is_name(fact.slot) || fact.value == WRITE_EMPTY_VALUE {
                continue;
            }
            let label = lessons.labels[fact.slot as usize];
            let value = value_toks.iter().find(|(slot, _)| *slot == fact.slot).map(|(_, tokens)| tokens.as_slice()).unwrap_or_default();
            if toks.contains(&label) || run_place(toks, value, 0).is_some() {
                out |= slot_bit(fact.slot);
            }
        }
        let mut present: Vec<u32> = Vec::new();
        let mut groups: Vec<(u32, [u32; WRITE_RULE_GROUP])> = Vec::new();
        self.read_groups(toks, &mut present, &mut groups);
        for (found_key, found_group) in groups {
            let key = self.keys[found_key as usize];
            if key.item != NO_NUMBER {
                if !given.items(key.slot).contains(&key.item) {
                    return None;
                }
                match items.iter_mut().find(|(slot, _)| *slot == key.slot) {
                    Some((_, seen)) => {
                        if !seen.contains(&key.item) {
                            seen.push(key.item);
                        }
                    }
                    None => items.push((key.slot, vec![key.item])),
                }
            } else {
                match given.value(key.slot) {
                    Some(value) if TextReader::in_group(&found_group, value) => out |= slot_bit(key.slot),
                    _ => return None,
                }
            }
        }
        for (slot, found) in &items {
            let mut all: Vec<u32> = given.items(*slot).to_vec();
            all.sort();
            all.dedup();
            if found.len() < all.len() {
                return None;
            }
            out |= slot_bit(*slot);
        }
        Some(out)
    }
}
