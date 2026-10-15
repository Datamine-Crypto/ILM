use crate::write::forms::{FormStore, WRITE_HEAD_AND_NEXT, WRITE_USUAL_RUN};
use crate::write::lessons::{WRITE_EMPTY_VALUE, WriterLessons};
use crate::base::first_sight::NumberTable;
use patterns::source;

pub struct UsualRuns;
source!(UsualRuns, "a form is usual when each of its runs of units stands in more than one lesson sentence; and units are alike when lesson sentences go on in the same ways after them");

impl FormStore {
    pub(crate) fn learn_values_with(&mut self, lessons: &WriterLessons) {
        self.values_with = vec![NumberTable::default(); lessons.slots.len()];
        for lesson in &lessons.rows {
            let mut units: Vec<u32> = lesson.units.iter().map(|unit| unit.sym).collect();
            units.sort();
            units.dedup();
            let mut seen: Vec<u32> = Vec::new();
            for (slot, _) in &lesson.facts {
                if seen.contains(slot) {
                    continue;
                }
                seen.push(*slot);
                let had = lesson.facts.iter().rev().find(|(other, _)| other == slot).map_or(WRITE_EMPTY_VALUE, |(_, value)| *value);
                if had == WRITE_EMPTY_VALUE {
                    continue;
                }
                for unit in &units {
                    *self.values_with[*slot as usize].entry(*unit).or_default().at(had) += 1;
                }
            }
        }
    }

    pub(crate) fn learn_runs(&mut self, lessons: &WriterLessons) {
        for form in &self.forms {
            let padded = FormStore::padded(lessons, &form.key);
            let last = padded.len().saturating_sub(WRITE_USUAL_RUN);
            for at in 0..=last {
                *self.usual_runs.entry(padded[at..(at + WRITE_USUAL_RUN).min(padded.len())].to_vec()).or_insert(0) += form.count;
            }
        }
        for at in 0..self.forms.len() {
            let usual = self.rarest(lessons, &self.forms[at].key);
            let rare = self.rare_runs(lessons, &self.forms[at].key);
            self.forms[at].usual = usual;
            self.forms[at].rare = rare;
        }
        for form in &self.forms {
            let mut key = FormStore::padded(lessons, &form.key);
            key.push(lessons.end);
            for at in 0..key.len() - WRITE_HEAD_AND_NEXT {
                *self.goes_on.entry(key[at]).or_default().at((key[at + 1], key[at + WRITE_HEAD_AND_NEXT])) += form.count;
            }
        }
        for (unit, ways) in &self.goes_on {
            let seen = self.second.entry(*unit).or_default();
            for ((_, second), count) in ways.rows() {
                *seen.at(*second) += *count;
            }
        }
        for form in &self.forms {
            let mut before = lessons.start;
            for unit in &form.key {
                *self.comes_after.entry(*unit).or_default().at(before) += form.count;
                if !lessons.label_slot.contains_key(unit) {
                    let next = self.next_units.entry(before).or_default();
                    if !next.contains(unit) {
                        next.push(*unit);
                    }
                }
                before = *unit;
            }
        }
        self.start_end = self.forms.iter().map(|form| FormStore::start_and_end(&form.key)).collect();
    }
}
