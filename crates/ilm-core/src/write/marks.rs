use crate::base::keys::NO_NUMBER;
use crate::base::symbols::Symbols;
use crate::write::check::{CheckRun, FactCheck, WRITE_WORD_FOR_WORD};
use crate::write::find::WantedValue;
use crate::write::rows::FactRow;
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag, NumberTable, all_counts, key_count};
use patterns::{because, source};

pub struct CountedMarks;
source!(CountedMarks, "the marks of the check of facts are counted from the lessons: a run of tokens is a mark of a slot when nearly all lesson texts that hold it have one of a small group of values of the slot");

pub const WRITE_CHECK_TEXTS: i64 = 20;
because!(WRITE_CHECK_TEXTS, CountedMarks, "the lesson texts that must hold a mark of the check, at least");

pub const WRITE_CHECK_SURE: f64 = 0.9;
because!(WRITE_CHECK_SURE, CountedMarks, "the share of the lesson texts with a mark whose facts hold one of the values of the mark; a higher share leaves more facts that the check cannot see");

pub const WRITE_NAME_TEXTS: i64 = 5;
because!(WRITE_NAME_TEXTS, CountedMarks, "the lesson texts that must hold a name, at least, for the check to count it as a name that a text may not add");

pub const WRITE_CHECK_GROUP: usize = 2;
because!(WRITE_CHECK_GROUP, CountedMarks, "the most values that one mark names: people use one wording for a pair of values of a slot; a mark also names at most half of the values of its slot");

pub const WRITE_CHECK_PARTS: usize = 2;
because!(WRITE_CHECK_PARTS, CountedMarks, "a slot holds lists when one part in this many of the items of its values with a cut stands in more than one value, and a mark names at most one part in this many of the values of its slot");

pub const WRITE_CAN_CHECK: f64 = 0.5;
because!(WRITE_CAN_CHECK, CountedMarks, "a fact can be checked when the check finds it in this share of the lesson texts that have it");

impl FactCheck {
    pub fn learn(lessons: &[FactRow], cut: Option<&str>) -> FactCheck {
        let mut check = FactCheck { cut: cut.map(str::to_string), slots: Symbols::new(), values: Symbols::new(), toks: Symbols::new(), list_slot: Vec::new(), slot_values: Vec::new(), word_for_word: Vec::new(), names: Vec::new(), marks: NumberTable::default(), can_check: NumberBag::default() };
        let raw: Vec<Vec<(u32, String)>> = lessons.iter().map(|row| row.facts.iter().map(|(slot, value)| (check.slots.number(slot), value.clone())).collect()).collect();
        let slots = check.slots.len();
        check.list_slot = vec![false; slots];
        if let Some(cut) = cut {
            let mut by_slot: Vec<Vec<&str>> = vec![Vec::new(); slots];
            for facts in &raw {
                for (slot, value) in facts {
                    if !by_slot[*slot as usize].contains(&value.as_str()) {
                        by_slot[*slot as usize].push(value.as_str());
                    }
                }
            }
            for (slot, values) in by_slot.iter().enumerate() {
                let lists: Vec<&str> = values.iter().copied().filter(|value| value.contains(cut)).collect();
                let mut inside: FirstSightCount<&str> = FirstSightCount::new();
                for value in values {
                    let mut once: Vec<&str> = Vec::new();
                    for item in value.split(cut) {
                        if !once.contains(&item) {
                            once.push(item);
                            *inside.at(item) += 1;
                        }
                    }
                }
                let mut items: Vec<&str> = lists.iter().flat_map(|value| value.split(cut)).collect();
                items.sort();
                items.dedup();
                let again = items.iter().filter(|item| key_count(&inside, item) > 1).count();
                check.list_slot[slot] = lists.len() > 1 && again * WRITE_CHECK_PARTS >= items.len();
            }
        }
        let units: Vec<Vec<(u32, String)>> = raw.iter().map(|facts| check.units(facts)).collect();
        check.slot_values = vec![FirstSightCount::new(); slots];
        let mut whole = vec![0i64; slots];
        let mut patterns: NumberTable<u32, WantedValue> = NumberTable::default();
        for (facts, row) in units.iter().zip(lessons) {
            for (slot, value) in facts {
                let number = check.values.number(value);
                *check.slot_values[*slot as usize].at(number) += 1;
                let pattern = patterns.entry(number).or_insert_with(|| WantedValue::new(value));
                whole[*slot as usize] += i64::from(pattern.stands(&row.text));
            }
        }
        check.word_for_word = (0..slots).map(|slot| (whole[slot] as f64) >= WRITE_WORD_FOR_WORD * (all_counts(&check.slot_values[slot]) as f64) && !check.slot_values[slot].rows().is_empty()).collect();
        let mut names: Vec<u32> = Vec::new();
        for slot in 0..slots {
            if !check.word_for_word[slot] {
                continue;
            }
            for (value, _) in check.slot_values[slot].rows() {
                if !check.values.name(*value).is_empty() && !names.contains(value) {
                    names.push(*value);
                }
            }
        }
        let mut stands = vec![0i64; names.len()];
        let mut given_there = vec![0i64; names.len()];
        for (facts, row) in units.iter().zip(lessons) {
            let low = row.text.to_lowercase();
            for (at, name) in names.iter().enumerate() {
                let text = check.values.name(*name);
                if patterns[name].stands_in_low(&row.text, &low) {
                    stands[at] += 1;
                    given_there[at] += i64::from(facts.iter().any(|(slot, value)| check.word_for_word[*slot as usize] && value == text));
                }
            }
        }
        for (at, name) in names.iter().enumerate() {
            if stands[at] >= WRITE_NAME_TEXTS && (given_there[at] as f64) >= WRITE_CHECK_SURE * (stands[at] as f64) {
                let text = check.values.name(*name).to_string();
                check.names.push((*name, WantedValue::new(&text), text));
            }
        }
        let mut rows: Vec<Vec<u32>> = Vec::with_capacity(lessons.len());
        for (facts, row) in units.iter().zip(lessons) {
            let tokens = check.token_row(facts, &row.text);
            rows.push(tokens.iter().map(|token| check.toks.number(token)).collect());
        }
        let mut count: NumberTable<CheckRun, i64> = NumberTable::default();
        let mut row_runs: Vec<Vec<CheckRun>> = Vec::with_capacity(rows.len());
        for toks in &rows {
            let mut runs: Vec<CheckRun> = FactCheck::runs_of(toks).into_iter().map(|(_, _, run)| run).collect();
            runs.sort();
            runs.dedup();
            for run in &runs {
                *count.entry(*run).or_insert(0) += 1;
            }
            row_runs.push(runs);
        }
        let mut with_fact: NumberTable<CheckRun, FirstSightTable<(u32, u32), i64>> = NumberTable::default();
        for (facts, runs) in units.iter().zip(&row_runs) {
            let mine: Vec<(u32, u32)> = facts.iter().filter(|(slot, _)| !check.word_for_word[*slot as usize]).map(|(slot, value)| (*slot, check.values.find(value).unwrap_or(NO_NUMBER))).collect();
            for run in runs {
                if count[run] < WRITE_CHECK_TEXTS {
                    continue;
                }
                let seen = with_fact.entry(*run).or_default();
                for fact in &mine {
                    *seen.at(*fact) += 1;
                }
            }
        }
        for (run, pairs) in &with_fact {
            let texts = count[run] as f64;
            let mut by_slot: Vec<(u32, Vec<(u32, i64)>)> = Vec::new();
            for ((slot, value), many) in pairs.rows() {
                match by_slot.iter_mut().find(|(seen, _)| seen == slot) {
                    Some((_, values)) => values.push((*value, *many)),
                    None => by_slot.push((*slot, vec![(*value, *many)])),
                }
            }
            let mut found: Vec<(u32, Vec<u32>)> = Vec::new();
            for (slot, mut values) in by_slot {
                values.sort_by(|one, other| other.1.cmp(&one.1));
                let mut group: Vec<u32> = Vec::new();
                let mut held = 0i64;
                for (value, many) in values {
                    if (held as f64) >= WRITE_CHECK_SURE * texts {
                        break;
                    }
                    group.push(value);
                    held += many;
                }
                if (held as f64) >= WRITE_CHECK_SURE * texts && group.len() * WRITE_CHECK_PARTS <= check.slot_values[slot as usize].rows().len() && group.len() <= WRITE_CHECK_GROUP {
                    found.push((slot, group));
                }
            }
            if !found.is_empty() {
                check.marks.insert(*run, found);
            }
        }
        let mut found: FirstSightTable<(u32, u32), (i64, i64)> = FirstSightTable::new();
        for (facts, toks) in units.iter().zip(&rows) {
            let told = check.told(toks);
            for (slot, value) in facts {
                if check.word_for_word[*slot as usize] {
                    continue;
                }
                let number = check.values.find(value).unwrap_or(NO_NUMBER);
                let seen = told.iter().find(|(known, _)| known == slot).is_some_and(|(_, groups)| groups.iter().any(|group| group.contains(&number)));
                let entry = found.at((*slot, number));
                entry.0 += 1;
                entry.1 += i64::from(seen);
            }
        }
        check.can_check = found.rows().iter().filter(|(_, (have, seen))| (*seen as f64) >= WRITE_CAN_CHECK * (*have as f64)).map(|(pair, _)| *pair).collect();
        check
    }
}
