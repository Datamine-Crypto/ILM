use crate::base::keys::{NO_NUMBER, padded};
use crate::base::symbols::Symbols;
use crate::know::signs::is_word_sign;
use crate::write::find::WantedValue;
use crate::write::marks::CountedMarks;
use crate::write::overlap::RunOverlap;
use crate::write::signs::low_tokens;
use crate::base::first_sight::{FirstSightCount, NumberBag, NumberTable};
use patterns::{because, source};

pub struct FactMarks;
source!(FactMarks, "a check of a text against its facts by marks counted from lessons, in the sense of the slot error rate that the field reports for texts from data: a given fact that the text does not say, a value that the text says and that was not given, and a fact that the text adds; a mark is a run of tokens that stands with a small group of values of one slot in nearly all of its lesson texts");

pub const WRITE_CHECK_RUN: usize = 4;
because!(WRITE_CHECK_RUN, FactMarks, "the most tokens in a mark of the check of facts");

pub const WRITE_WORD_FOR_WORD: f64 = 0.98;
because!(WRITE_WORD_FOR_WORD, FactMarks, "a slot is word for word when this share of its lesson texts holds its value as it is");

pub const WRITE_PLACE_WORD: &str = "placeof";
because!(WRITE_PLACE_WORD, FactMarks, "the start of the one token that stands in a text for a given value of a word for word slot, with the word signs of the slot after it, so that no name is a part of a mark");

pub const WRITE_UNSEEN_TOKEN: u32 = u32::MAX - 1;
because!(WRITE_UNSEEN_TOKEN, FactMarks, "the number of a token that no lesson text held: it is not the mark of no number, which fills the key of a shorter run, so a run with such a token is the key of no mark");

pub type CheckRun = [u32; WRITE_CHECK_RUN];
because!(CheckRun, FactMarks, RunOverlap, "a run of tokens as one key of a fixed width, with the mark of no number in the places that a shorter run leaves");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactFault {
    Lost,
    Wrong,
    Added,
}
because!(FactFault, FactMarks, "what the check of facts can find in a text: a given fact that no mark of the text says, marks of a given slot that all name another value, and a mark of a slot or a name that the facts do not have");

pub struct FactCheck {
    pub(crate) cut: Option<String>,
    pub(crate) slots: Symbols,
    pub(crate) values: Symbols,
    pub(crate) toks: Symbols,
    pub(crate) list_slot: Vec<bool>,
    pub(crate) slot_values: Vec<FirstSightCount<u32>>,
    pub word_for_word: Vec<bool>,
    pub names: Vec<(u32, WantedValue, String)>,
    pub marks: NumberTable<CheckRun, Vec<(u32, Vec<u32>)>>,
    pub(crate) can_check: NumberBag<(u32, u32)>,
}
because!(FactCheck, FactMarks, CountedMarks, "the check of facts of the measure, which no writer makes: does a text say every fact, no other value, and no fact that was not given. It is counted from all lessons of the learn lists. A slot whose value stands in nearly every lesson text as it is must stand in a text as it is, and a text may hold no other name. For the other slots a mark is a run of tokens that stands in enough lesson texts with a small group of values of one slot; a mark inside a longer mark of the same text does not count. A fact can be checked when the check finds it in enough of the lesson texts that have it; a fact that cannot be checked is never counted as lost. learn makes the check; faults gives what it finds in a text for a list of facts; cannot says for a list how many of its facts it cannot check");

impl FactCheck {
    pub(crate) fn units(&self, facts: &[(u32, String)]) -> Vec<(u32, String)> {
        let mut out: Vec<(u32, String)> = Vec::new();
        for (slot, value) in facts {
            match &self.cut {
                Some(cut) if self.list_slot.get(*slot as usize).copied().unwrap_or(false) => out.extend(value.split(cut.as_str()).map(|item| (*slot, item.to_string()))),
                _ => out.push((*slot, value.clone())),
            }
        }
        out
    }

    pub(crate) fn place(slot: &str) -> String {
        let kept: String = slot.chars().filter(|sign| is_word_sign(*sign)).collect();
        format!(" {WRITE_PLACE_WORD}{} ", kept.to_lowercase())
    }

    pub(crate) fn token_row(&self, units: &[(u32, String)], text: &str) -> Vec<String> {
        let mut text = text.to_string();
        for (slot, value) in units {
            if !value.is_empty() && self.word_for_word.get(*slot as usize).copied().unwrap_or(false) {
                text = WantedValue::new(value).replaced(&text, &FactCheck::place(self.slots.name(*slot)));
            }
        }
        low_tokens(&text)
    }

    pub(crate) fn runs_of(toks: &[u32]) -> Vec<(usize, usize, CheckRun)> {
        let mut out: Vec<(usize, usize, CheckRun)> = Vec::new();
        for size in 1..=WRITE_CHECK_RUN {
            for (at, run) in toks.windows(size).enumerate() {
                out.push((at, at + size, padded(run)));
            }
        }
        out
    }

    pub(crate) fn told(&self, toks: &[u32]) -> Vec<(u32, Vec<&Vec<u32>>)> {
        let found: Vec<(usize, usize, CheckRun)> = FactCheck::runs_of(toks).into_iter().filter(|(_, _, run)| self.marks.contains_key(run)).collect();
        let mut told: Vec<(u32, Vec<&Vec<u32>>)> = Vec::new();
        for (from, to, run) in &found {
            if found.iter().any(|(a, b, _)| a <= from && to <= b && b - a > to - from) {
                continue;
            }
            for (slot, values) in &self.marks[run] {
                match told.iter_mut().find(|(seen, _)| seen == slot) {
                    Some((_, groups)) => groups.push(values),
                    None => told.push((*slot, vec![values])),
                }
            }
        }
        told
    }

    fn numbered(&self, toks: &[String]) -> Vec<u32> {
        toks.iter().map(|token| self.toks.find(token).unwrap_or(WRITE_UNSEEN_TOKEN)).collect()
    }

    pub(crate) fn facts_of(&self, facts: &[(String, String)]) -> Vec<(u32, String)> {
        let known = self.slots.len() as u32;
        let mut fresh: Vec<&str> = Vec::new();
        let raw: Vec<(u32, String)> = facts
            .iter()
            .map(|(slot, value)| {
                let number = self.slots.find(slot).unwrap_or_else(|| {
                    let place = fresh.iter().position(|seen| *seen == slot.as_str()).unwrap_or_else(|| {
                        fresh.push(slot.as_str());
                        fresh.len() - 1
                    });
                    known + place as u32
                });
                (number, value.clone())
            })
            .collect();
        self.units(&raw)
    }

    pub fn faults(&self, facts: &[(String, String)], text: &str) -> Vec<FactFault> {
        let units = self.facts_of(facts);
        let word_for_word = |slot: u32| self.word_for_word.get(slot as usize).copied().unwrap_or(false);
        let mut out: Vec<FactFault> = Vec::new();
        let mut given: Vec<(u32, Vec<&str>)> = Vec::new();
        for (slot, value) in &units {
            match given.iter_mut().find(|(seen, _)| seen == slot) {
                Some((_, values)) => {
                    if !values.contains(&value.as_str()) {
                        values.push(value.as_str());
                    }
                }
                None => given.push((*slot, vec![value.as_str()])),
            }
        }
        let own: Vec<&str> = units.iter().filter(|(slot, _)| word_for_word(*slot)).map(|(_, value)| value.as_str()).collect();
        for (slot, value) in &units {
            if word_for_word(*slot) && !value.is_empty() && !WantedValue::new(value).stands(text) {
                out.push(FactFault::Lost);
            }
        }
        let low_own: Vec<String> = own.iter().map(|value| value.to_lowercase()).collect();
        for (_, pattern, name) in &self.names {
            let low = name.to_lowercase();
            if !own.contains(&name.as_str()) && !low_own.iter().any(|value| value.contains(low.as_str())) && pattern.stands(text) {
                out.push(FactFault::Added);
            }
        }
        let toks = self.numbered(&self.token_row(&units, text));
        let told = self.told(&toks);
        for (slot, values) in &given {
            if word_for_word(*slot) {
                continue;
            }
            let numbers: Vec<u32> = values.iter().map(|value| self.values.find(value).unwrap_or(NO_NUMBER)).collect();
            let groups: &[&Vec<u32>] = told.iter().find(|(seen, _)| seen == slot).map_or(&[], |(_, groups)| groups.as_slice());
            if !groups.is_empty() && !groups.iter().any(|group| numbers.iter().any(|number| *number != NO_NUMBER && group.contains(number))) {
                out.push(FactFault::Wrong);
                continue;
            }
            for number in &numbers {
                if self.can_check.contains(&(*slot, *number)) && !groups.iter().any(|group| group.contains(number)) {
                    out.push(FactFault::Lost);
                }
            }
        }
        for (slot, _) in &told {
            if !given.iter().any(|(seen, _)| seen == slot) {
                out.push(FactFault::Added);
            }
        }
        out
    }

    pub fn cannot(&self, facts: &[(String, String)]) -> (usize, usize) {
        let units = self.facts_of(facts);
        let blind = units.iter().filter(|(slot, value)| !self.word_for_word.get(*slot as usize).copied().unwrap_or(false) && !self.can_check.contains(&(*slot, self.values.find(value).unwrap_or(NO_NUMBER)))).count();
        (blind, units.len())
    }

    pub fn mark_count(&self) -> usize {
        self.marks.values().map(Vec::len).sum()
    }
}
