use crate::write::find::WantedValue;
use crate::write::measure::grouped_lists;
use crate::write::rows::FactRow;
use patterns::{because, source};

pub struct LookUpLine;
source!(LookUpLine, "a writer is measured against a line that learns nothing: the text of the lesson with the nearest list of facts, with the values changed; what a writer gives above that line is what it learned");

pub struct NearestLesson {
    pub(crate) lists: Vec<(Vec<(String, String)>, String)>,
}
because!(NearestLesson, LookUpLine, "the line that a writer must beat, which learns nothing: for a list of facts it gives the text of the lesson with the nearest list of facts, with every value changed that stands in it word for word. A writer that is not above it has learned no more than a look-up. learn keeps the first text of every list of facts of the lessons; write gives the text");

impl NearestLesson {
    pub fn learn(lessons: &[FactRow]) -> NearestLesson {
        NearestLesson { lists: grouped_lists(lessons).into_iter().map(|list| (list.facts, list.references.into_iter().next().unwrap_or_default())).collect() }
    }

    fn value_of<'b>(facts: &'b [(String, String)], slot: &str) -> Option<&'b str> {
        facts.iter().rev().find(|(seen, _)| seen == slot).map(|(_, value)| value.as_str())
    }

    pub fn write(&self, facts: &[(String, String)]) -> String {
        let mut best: Option<(&(Vec<(String, String)>, String), (i64, i64))> = None;
        for list in &self.lists {
            let have = &list.0;
            let mut slots: Vec<&str> = facts.iter().map(|(slot, _)| slot.as_str()).chain(have.iter().map(|(slot, _)| slot.as_str())).collect();
            slots.sort();
            slots.dedup();
            let differs = slots.iter().filter(|slot| NearestLesson::value_of(facts, slot).is_some() != NearestLesson::value_of(have, slot).is_some()).count() as i64;
            let same = slots.iter().filter(|slot| NearestLesson::value_of(facts, slot).is_some() && NearestLesson::value_of(facts, slot) == NearestLesson::value_of(have, slot)).count() as i64;
            let score = (-differs, same);
            if best.as_ref().is_none_or(|(_, top)| score > *top) {
                best = Some((list, score));
            }
        }
        let Some(((have, text), _)) = best else {
            return String::new();
        };
        let mut text = text.clone();
        let mut done: Vec<&str> = Vec::new();
        for (slot, _) in facts {
            if done.contains(&slot.as_str()) {
                continue;
            }
            done.push(slot.as_str());
            let value = NearestLesson::value_of(facts, slot).unwrap_or_default();
            if let Some(was) = NearestLesson::value_of(have, slot) {
                if was != value {
                    text = WantedValue::new(was).replaced(&text, value);
                }
            }
        }
        text
    }
}
