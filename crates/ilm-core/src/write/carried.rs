use crate::base::keys::{NO_NUMBER, padded};
use crate::write::lessons::{WRITE_EMPTY_VALUE, WRITE_LONGEST_RUN, WriterLessons};
use crate::write::reader::{ReadFound, ReadKey, TextReader};
use crate::base::first_sight::FirstSightTable;
use patterns::{because, source};

pub struct CarriedSlots;
source!(CarriedSlots, "a fact that no words of a text say is carried by the choice of the form: a slot that shapes the whole text, such as the sort of a sentence, and a slot whose facts the reader finds in too few of its lessons; a form of a lesson with such a fact serves only lists of facts with the same value");

pub const WRITE_CAN_READ_SHARE: f64 = 0.5;
because!(WRITE_CAN_READ_SHARE, CarriedSlots, "a slot is carried by the choice of the form when the reader finds its facts in under this share of their lessons");

pub const WRITE_WHOLE_SHARE: f64 = 0.5;
because!(WRITE_WHOLE_SHARE, CarriedSlots, "a slot shapes the whole text when this share of the tokens of its lesson sentences stand in runs that belong to it");

impl TextReader {
    pub(crate) fn shares(&mut self, lessons: &WriterLessons) -> (FirstSightTable<(Vec<u32>, u32), (i64, i64)>, Vec<u32>) {
        let slots = lessons.slots.len();
        let mut mine = vec![0i64; slots];
        let mut every = vec![0i64; slots];
        for (sentence, toks) in self.sent_toks.iter().enumerate() {
            let mut under: Vec<Vec<bool>> = vec![Vec::new(); slots];
            for size in 1..=WRITE_LONGEST_RUN {
                if toks.len() < size {
                    break;
                }
                for at in 0..=toks.len() - size {
                    let run = self.run_of[&padded(&toks[at..at + size])];
                    if !self.marked(run) || self.owner[run as usize] == NO_NUMBER {
                        continue;
                    }
                    let slot = self.key_slot(self.owner[run as usize]) as usize;
                    if under[slot].is_empty() {
                        under[slot] = vec![false; toks.len()];
                    }
                    for place in at..at + size {
                        under[slot][place] = true;
                    }
                }
            }
            let lesson = &lessons.rows[self.sent_lesson[sentence] as usize];
            for fact in &lesson.given.facts {
                if !lessons.is_name(fact.slot) && fact.value != WRITE_EMPTY_VALUE {
                    every[fact.slot as usize] += toks.len() as i64;
                    mine[fact.slot as usize] += under[fact.slot as usize].iter().filter(|held| **held).count() as i64;
                }
            }
        }
        self.whole_share = (0..slots).map(|slot| (every[slot] > 0).then(|| (mine[slot] as f64) / (every[slot] as f64))).collect();
        self.carried = self.whole_share.iter().map(|share| share.is_some_and(|share| share >= WRITE_WHOLE_SHARE)).collect();
        if self.carried.iter().any(|carried| *carried) {
            let keys: Vec<u32> = (0..self.keys.len() as u32).filter(|key| !self.carried[self.key_slot(*key) as usize]).collect();
            self.own_and_cover(lessons, &keys);
        }
        self.carried_order = (0..slots as u32).filter(|slot| self.carried[*slot as usize]).collect();
        self.carried_order.sort_by(|one, other| lessons.slots.name(*one).cmp(lessons.slots.name(*other)));
        let mut found = vec![0i64; slots];
        let mut have = vec![0i64; slots];
        let mut inside: FirstSightTable<(Vec<u32>, u32), (i64, i64)> = FirstSightTable::new();
        for (number, lesson) in lessons.rows.iter().enumerate() {
            let (first, last) = self.lesson_sents[number];
            let read: Vec<Vec<ReadFound>> = (first..last).map(|sentence| self.read(&self.sent_toks[sentence as usize])).collect();
            let part: Vec<u32> = self.carried_order.iter().map(|slot| lesson.given.value(*slot).unwrap_or(NO_NUMBER)).collect();
            for fact in &lesson.given.facts {
                let slot = fact.slot;
                if lessons.is_name(slot) || self.carried[slot as usize] || fact.value == WRITE_EMPTY_VALUE {
                    continue;
                }
                let got = if lessons.is_list(slot) {
                    let mut all: Vec<u32> = fact.items.clone();
                    all.sort();
                    all.dedup();
                    read.iter().zip(&lesson.sents).any(|(seen, (from, to))| {
                        let mut here: Vec<u32> = seen.iter().map(|found| self.keys[found.key as usize]).filter(|key| key.slot == slot && key.item != NO_NUMBER).map(|key| key.item).collect();
                        here.extend(lesson.units[*from as usize..*to as usize].iter().filter(|unit| unit.placed_item() && unit.slot == slot).map(|unit| unit.item));
                        here.sort();
                        here.dedup();
                        here == all
                    })
                } else {
                    read.iter().any(|seen| seen.iter().any(|found| self.keys[found.key as usize] == ReadKey { slot, item: NO_NUMBER } && TextReader::in_group(&found.group, fact.value)))
                };
                have[slot as usize] += 1;
                found[slot as usize] += i64::from(got);
                let entry = inside.at((part.clone(), slot));
                entry.0 += 1;
                entry.1 += i64::from(got);
            }
        }
        self.found_share = (0..slots).map(|slot| (have[slot] > 0).then(|| (found[slot] as f64) / (have[slot] as f64))).collect();
        let unread: Vec<u32> = (0..slots as u32).filter(|slot| self.found_share[*slot as usize].is_some_and(|share| share < WRITE_CAN_READ_SHARE)).collect();
        (inside, unread)
    }
}
