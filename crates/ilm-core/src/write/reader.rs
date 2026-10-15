use crate::base::keys::{NO_NUMBER, padded};
use crate::write::carried::CarriedSlots;
use crate::write::carried::WRITE_CAN_READ_SHARE;
use crate::write::cover::CoveringRules;
use crate::write::lessons::{WRITE_EMPTY_VALUE, WRITE_LONGEST_RUN, WriterLessons, slot_bit};
use crate::write::owners::RunOwners;
use crate::write::owners::WRITE_RULE_GROUP;
use crate::write::read::SentenceReading;
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag, NumberTable};
use patterns::{because, source};
use std::hash::Hash;

pub struct DecisionListReader;
source!(DecisionListReader, "a decision list in the sense of Rivest: an ordered list of tests of which the first that holds decides. It is learned by covering, as the rule learners of Michalski and of Clark and Niblett do: the sure test with the most texts is taken, its texts are taken away, and the rest is counted again. A test is a run of tokens, and it is sure by the bound of Chernoff on the count of its texts");

pub const WRITE_RULE_TEXTS: i64 = 4;
because!(WRITE_RULE_TEXTS, DecisionListReader, "the lesson texts that must hold the run of a rule of the reader, at least");

pub const WRITE_PART_LESSONS: i64 = 10;
because!(WRITE_PART_LESSONS, DecisionListReader, "the lessons of one sort of text with a slot, for the slot to be judged inside that sort; and the lessons with a slot for the slot to have a word of its own");

pub const WRITE_WORD_SHARE: f64 = 0.95;
because!(WRITE_WORD_SHARE, DecisionListReader, "the word of a slot stands in this share of the lesson texts with the slot, and this share of the lesson texts that hold it have the slot");

pub const WRITE_AS_IT_IS_SHARE: f64 = 0.3;
because!(WRITE_AS_IT_IS_SHARE, DecisionListReader, "a value is written as it is when this share of its lesson texts holds it word for word");

pub type ReadRun = [u32; WRITE_LONGEST_RUN];
because!(ReadRun, DecisionListReader, "a run of units as one key of a fixed width: its unit numbers, and the mark of no number in the places that a shorter run leaves");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReadKey {
    pub slot: u32,
    pub item: u32,
}
because!(ReadKey, DecisionListReader, "what a list of rules of the reader reads: a slot, or one item of a slot that holds lists, for the lessons where the item stands in other words than its own");

#[derive(Clone, Debug, PartialEq)]
pub struct ReadRule {
    pub run: u32,
    pub group: [u32; WRITE_RULE_GROUP],
    pub had: i64,
    pub held: i64,
    pub finer: Vec<ReadRule>,
}
because!(ReadRule, DecisionListReader, "one rule of the reader: a run of tokens, the group of values that it names, the lesson texts with the run, those of them with a value of the group, and the finer rules that hold inside its sentences");

#[derive(Clone, Debug, PartialEq)]
pub struct ReadFound {
    pub key: u32,
    pub group: [u32; WRITE_RULE_GROUP],
    pub spans: Vec<(u32, u32)>,
}
because!(ReadFound, DecisionListReader, "what the reader finds in one sentence for one key: the group of values of the rule that reads it, and where the runs of the rules stand");

#[derive(Clone, Debug, Default)]
pub struct TextReader {
    pub sent_lesson: Vec<u32>,
    pub sent_toks: Vec<Vec<u32>>,
    pub lesson_sents: Vec<(u32, u32)>,
    pub run_of: NumberTable<ReadRun, u32>,
    pub run_keys: Vec<ReadRun>,
    pub count: Vec<i64>,
    pub lesson_runs: Vec<Vec<u32>>,
    pub per: Vec<Vec<u32>>,
    pub holds: Vec<Vec<u32>>,
    pub run_order: Vec<u32>,
    pub keys: Vec<ReadKey>,
    pub value_of: Vec<Vec<u32>>,
    pub key_values: Vec<FirstSightCount<u32>>,
    pub word_of: Vec<u32>,
    pub owner: Vec<u32>,
    pub rules: Vec<Vec<ReadRule>>,
    pub read_keys: Vec<bool>,
    pub top: Vec<(u32, u32)>,
    pub whole_share: Vec<Option<f64>>,
    pub found_share: Vec<Option<f64>>,
    pub carried: Vec<bool>,
    pub carried_order: Vec<u32>,
    pub plain: Vec<u32>,
    pub plain_needs: u64,
    pub carried_in: FirstSightTable<Vec<u32>, u64>,
    pub as_it_is: NumberTable<Vec<u32>, Vec<u32>>,
}
because!(TextReader, DecisionListReader, RunOwners, CoveringRules, SentenceReading, CarriedSlots, "the reader of the writer: a decision list for each slot and for each item of a list in other words than its own. Each run of tokens inside a sentence belongs to the key that its texts are tied to most; a run is a rule when most of its texts have one small group of values and chance does not give that count; the sure run with the most texts of its group is taken, its sentences are taken away, and the rest is counted again; inside the sentences of a rule the same count gives its finer rules. Beside the rules it keeps what the reader shows about the slots: the word of a slot, the slots that shape the whole text or that the reader cannot find and that are carried by the choice of the form, the plain sort of text, and the values that people write as they are. learn makes it from the lessons; read gives what stands in one sentence; said gives the slots that a sentence says for a list of facts, or nothing when it says a thing that the list does not hold");

pub(crate) fn run_units(key: &ReadRun) -> usize {
    key.iter().take_while(|unit| **unit != NO_NUMBER).count()
}
because!(run_units, DecisionListReader, "a key of a fixed width holds no length, so the length is the number of places before the first mark of no number; of sure tests with the same count the shorter is tried first");


impl TextReader {
    pub fn key_slot(&self, key: u32) -> u32 {
        self.keys[key as usize].slot
    }

    pub fn run_number(&self, units: &[u32]) -> Option<u32> {
        self.run_of.get(&padded(units)).copied()
    }

    pub fn marked(&self, run: u32) -> bool {
        self.count[run as usize] >= WRITE_RULE_TEXTS
    }

    pub fn in_group(group: &[u32; WRITE_RULE_GROUP], value: u32) -> bool {
        value != NO_NUMBER && group.contains(&value)
    }

    fn count_runs(&mut self, lessons: &WriterLessons) {
        for (number, lesson) in lessons.rows.iter().enumerate() {
            let first = self.sent_toks.len() as u32;
            let mut mine: Vec<u32> = Vec::new();
            for (from, to) in &lesson.sents {
                let toks: Vec<u32> = lesson.units[*from as usize..*to as usize].iter().map(|unit| unit.sym).collect();
                for size in 1..=WRITE_LONGEST_RUN {
                    if toks.len() < size {
                        break;
                    }
                    for at in 0..=toks.len() - size {
                        let key = padded(&toks[at..at + size]);
                        let fresh = self.run_keys.len() as u32;
                        let run = *self.run_of.entry(key).or_insert(fresh);
                        if run == fresh {
                            self.run_keys.push(key);
                        }
                        mine.push(run);
                    }
                }
                self.sent_lesson.push(number as u32);
                self.sent_toks.push(toks);
            }
            mine.sort();
            mine.dedup();
            self.lesson_runs.push(mine);
            self.lesson_sents.push((first, self.sent_toks.len() as u32));
        }
        self.count = vec![0; self.run_keys.len()];
        for mine in &self.lesson_runs {
            for run in mine {
                self.count[*run as usize] += 1;
            }
        }
        self.holds = vec![Vec::new(); self.run_keys.len()];
        for (sentence, toks) in self.sent_toks.iter().enumerate() {
            let mut found: Vec<u32> = Vec::new();
            for size in 1..=WRITE_LONGEST_RUN {
                if toks.len() < size {
                    break;
                }
                for at in 0..=toks.len() - size {
                    let run = self.run_of[&padded(&toks[at..at + size])];
                    if self.count[run as usize] >= WRITE_RULE_TEXTS {
                        found.push(run);
                    }
                }
            }
            found.sort();
            found.dedup();
            for run in &found {
                self.holds[*run as usize].push(sentence as u32);
            }
            self.per.push(found);
        }
        let mut marked: Vec<u32> = (0..self.run_keys.len() as u32).filter(|run| self.count[*run as usize] >= WRITE_RULE_TEXTS).collect();
        let ranked = |run: &u32| -> [u32; WRITE_LONGEST_RUN] {
            let mut ranks = [0u32; WRITE_LONGEST_RUN];
            for (place, unit) in self.run_keys[*run as usize].iter().enumerate() {
                if *unit != NO_NUMBER {
                    ranks[place] = lessons.rank[*unit as usize] + 1;
                }
            }
            ranks
        };
        marked.sort_by_key(ranked);
        self.run_order = vec![NO_NUMBER; self.run_keys.len()];
        for (place, run) in marked.iter().enumerate() {
            self.run_order[*run as usize] = place as u32;
        }
    }

    fn make_keys(&mut self, lessons: &WriterLessons) {
        let total = lessons.rows.len();
        for slot in 0..lessons.slots.len() as u32 {
            if lessons.is_name(slot) {
                continue;
            }
            if lessons.is_list(slot) {
                let mut items: Vec<u32> = lessons.rows.iter().flat_map(|lesson| lesson.given.items(slot).iter().copied()).collect();
                items.sort_by_key(|item| lessons.value_rank[*item as usize]);
                items.dedup();
                for item in items {
                    let mut values = vec![NO_NUMBER; total];
                    for (number, lesson) in lessons.rows.iter().enumerate() {
                        let placed = lesson.units.iter().any(|unit| unit.slot == slot && unit.item == item);
                        if lesson.given.items(slot).contains(&item) && !placed {
                            values[number] = item;
                        }
                    }
                    self.keys.push(ReadKey { slot, item });
                    self.value_of.push(values);
                }
            } else {
                let values = lessons.rows.iter().map(|lesson| lesson.given.value(slot).filter(|value| *value != WRITE_EMPTY_VALUE).unwrap_or(NO_NUMBER)).collect();
                self.keys.push(ReadKey { slot, item: NO_NUMBER });
                self.value_of.push(values);
            }
        }
        for values in &self.value_of {
            let mut base: FirstSightCount<u32> = FirstSightCount::new();
            for value in values {
                if *value != NO_NUMBER {
                    *base.at(*value) += 1;
                }
            }
            self.key_values.push(base);
        }
    }

    fn find_words(&mut self, lessons: &WriterLessons) {
        let slots = lessons.slots.len();
        self.word_of = vec![NO_NUMBER; slots];
        let label: NumberBag<u32> = lessons.labels.iter().copied().collect();
        for slot in 0..slots as u32 {
            if lessons.is_name(slot) {
                continue;
            }
            let mut with_slot = 0i64;
            let mut with_both: FirstSightCount<u32> = FirstSightCount::new();
            for (number, lesson) in lessons.rows.iter().enumerate() {
                if !lesson.given.has(slot) {
                    continue;
                }
                with_slot += 1;
                for run in &self.lesson_runs[number] {
                    let key = &self.run_keys[*run as usize];
                    if run_units(key) == 1 && !label.contains(&key[0]) {
                        *with_both.at(key[0]) += 1;
                    }
                }
            }
            let mut best: Option<(f64, u32)> = None;
            for (token, both) in with_both.rows() {
                let alone = self.count[self.run_of[&padded(&[*token])] as usize];
                if with_slot >= WRITE_PART_LESSONS && (*both as f64) >= WRITE_WORD_SHARE * (with_slot as f64) && (*both as f64) >= WRITE_WORD_SHARE * (alone as f64) {
                    let score = (*both as f64) / (with_slot as f64) * (*both as f64) / (alone as f64);
                    let better = best.is_none_or(|(top, word)| score > top || (score == top && lessons.rank[*token as usize] > lessons.rank[word as usize]));
                    if better {
                        best = Some((score, *token));
                    }
                }
            }
            if let Some((_, token)) = best {
                self.word_of[slot as usize] = token;
            }
        }
    }

    pub fn learn(lessons: &WriterLessons) -> TextReader {
        let mut reader = TextReader::default();
        reader.count_runs(lessons);
        reader.make_keys(lessons);
        reader.find_words(lessons);
        let keys: Vec<u32> = (0..reader.keys.len() as u32).collect();
        reader.own_and_cover(lessons, &keys);
        let (inside, unread) = reader.shares(lessons);
        if !unread.is_empty() {
            for slot in &unread {
                reader.carried[*slot as usize] = true;
            }
            for key in 0..reader.keys.len() {
                if unread.contains(&reader.keys[key].slot) {
                    reader.read_keys[key] = false;
                    reader.rules[key] = Vec::new();
                }
            }
            reader.set_top();
        }
        let slots = lessons.slots.len();
        reader.plain = vec![NO_NUMBER; slots];
        for slot in &reader.carried_order {
            let mut seen: FirstSightCount<u32> = FirstSightCount::new();
            for lesson in &lessons.rows {
                if lesson.given.has(*slot) {
                    *seen.at(lesson.given.value(*slot).unwrap_or(WRITE_EMPTY_VALUE)) += 1;
                }
            }
            reader.plain[*slot as usize] = seen.first_most().map(|(key, _)| key).copied().unwrap_or(NO_NUMBER);
        }
        let mut plain_lessons = 0i64;
        let mut has = vec![0i64; slots];
        for lesson in &lessons.rows {
            let plain = !reader.carried_order.is_empty() && reader.carried_order.iter().all(|slot| lesson.given.value(*slot) == Some(reader.plain[*slot as usize]));
            if plain {
                plain_lessons += 1;
                for fact in &lesson.given.facts {
                    if fact.value != WRITE_EMPTY_VALUE {
                        has[fact.slot as usize] += 1;
                    }
                }
            }
        }
        for slot in 0..slots as u32 {
            if has[slot as usize] > 0 && lessons.is_name(slot) && (has[slot as usize] as f64) >= WRITE_WORD_SHARE * (plain_lessons as f64) {
                reader.plain_needs |= slot_bit(slot);
            }
        }
        if !reader.carried_order.is_empty() {
            for ((part, slot), (have, found)) in inside.rows() {
                if *have >= WRITE_PART_LESSONS && !reader.carried[*slot as usize] && (*found as f64) < WRITE_CAN_READ_SHARE * (*have as f64) {
                    *reader.carried_in.at(part.clone()) |= slot_bit(*slot);
                }
            }
        }
        let mut have: FirstSightTable<(u32, u32), (i64, i64)> = FirstSightTable::new();
        for (number, lesson) in lessons.rows.iter().enumerate() {
            for (slot, value) in &lesson.facts {
                if *value != WRITE_EMPTY_VALUE && !lessons.is_name(*slot) && !reader.carried[*slot as usize] && !lessons.is_list(*slot) {
                    let tokens = &lessons.value_tokens[*value as usize];
                    let stands = tokens.len() <= WRITE_LONGEST_RUN && !tokens.is_empty() && reader.run_number(tokens).is_some_and(|run| reader.lesson_runs[number].binary_search(&run).is_ok());
                    let entry = have.at((*slot, *value));
                    entry.0 += 1;
                    entry.1 += i64::from(stands);
                }
            }
        }
        for ((slot, value), (all, as_it_is)) in have.rows() {
            let tokens = &lessons.value_tokens[*value as usize];
            if !tokens.is_empty() && (*as_it_is as f64) >= WRITE_AS_IT_IS_SHARE * (*all as f64) && tokens.len() <= WRITE_LONGEST_RUN {
                let slots = reader.as_it_is.entry(tokens.clone()).or_default();
                if !slots.contains(slot) {
                    slots.push(*slot);
                }
            }
        }
        reader
    }
}
