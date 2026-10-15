use crate::base::keys::NO_NUMBER;
use crate::base::sums::compensated;
use crate::write::clean::CleanLessons;
use crate::write::groups::FormGroups;
use crate::write::joins::JoinedLessonForms;
use crate::write::lessons::{WriterLessons, slot_bit};
use crate::write::phrases::ListWords;
use crate::write::reader::TextReader;
use crate::write::runs::UsualRuns;
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag, NumberTable, all_counts, key_count};
use patterns::{because, source};

pub struct TemplateForms;
source!(TemplateForms, "writing from facts with templates of sentences, the way that Reiter and Dale describe for systems that write from data: a template is a sentence with a place for each value. Here every template is counted from lessons: a sentence of a lesson whose text the reader reads back to exactly its facts, with a place for each value that stands word for word; templates are joined at a cut where the units around the cut agree");

pub const WRITE_USUAL_SENTENCES: i64 = 2;
because!(WRITE_USUAL_SENTENCES, TemplateForms, "a form is usual when each of its runs of units stands in this many lesson sentences or more");

pub const WRITE_USUAL_RUN: usize = 4;
because!(WRITE_USUAL_RUN, TemplateForms, "the units of the runs by which a form is judged usual or rare");

pub const WRITE_ALIKE_SHARE: f64 = 0.25;
because!(WRITE_ALIKE_SHARE, TemplateForms, "two units are alike when they share this much of the ways in which lesson sentences go on after them, or of the units before them");

pub const WRITE_OPEN_UNITS: usize = 2;
because!(WRITE_OPEN_UNITS, TemplateForms, "the first units of a lesson sentence by which a new sentence is said to start like it");

pub const WRITE_SAME_UNITS: usize = 2;
because!(WRITE_SAME_UNITS, TemplateForms, "the units before the cut that are the same in the lesson of the head and the lesson of the tail of a joined form");

pub const WRITE_LIST_GAP: usize = 2;
because!(WRITE_LIST_GAP, TemplateForms, "the most units between neighbouring items of a list in one phrase: a sign and a joining word");

pub const WRITE_INNER_ITEM: u32 = 0;
because!(WRITE_INNER_ITEM, TemplateForms, "the position of a place for an item of a list that is neither the first nor the last");

pub const WRITE_FIRST_ITEM: u32 = 1;
because!(WRITE_FIRST_ITEM, TemplateForms, "the position of the place for the first item of a list of several");

pub const WRITE_LAST_ITEM: u32 = 2;
because!(WRITE_LAST_ITEM, TemplateForms, "the position of the place for the last item of a list of several");

pub const WRITE_ONLY_ITEM: u32 = 3;
because!(WRITE_ONLY_ITEM, TemplateForms, "the position of the place for the only item of a list, and of every place of a value that is no item of a list");

pub type BoundKey = Vec<(u32, Vec<u32>)>;
because!(BoundKey, TemplateForms, "the values that a form is bound to, as one key: each slot once in the order of the slots with its values in their order, so that two forms bound to the same values have the same key");

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SentenceForm {
    pub key: Vec<u32>,
    pub count: i64,
    pub first: i64,
    pub later: i64,
    pub places: Vec<(u32, u32)>,
    pub slots: u64,
    pub fixed: Vec<(u32, Vec<u32>)>,
    pub said_at: Vec<(u32, Vec<(u32, u32)>)>,
    pub tied: Vec<(u32, u32)>,
    pub sign: u32,
    pub which: Vec<u32>,
    pub parts: Option<(u32, u32, u32, u32)>,
    pub resized: Option<(u32, Vec<(u32, u32)>)>,
    pub edited: Option<(u32, Vec<u32>, Vec<u32>)>,
    pub needs: Vec<(u32, Vec<u32>)>,
    pub after: Vec<(u32, Vec<u32>)>,
    pub usual: i64,
    pub rare: i64,
    pub lesson: (u32, u32),
}
because!(SentenceForm, TemplateForms, "a form of sentence: its units with a label in each place; how many lesson sentences have it, as the first sentence of a text and as a later one; its places with their slots; the slots that it says; the values of the lessons whose fixed words say a slot in it; where it says each slot; the facts that no words of it say and that it is bound to; the number of the item for each place of a list; for a joined form its head and tail, for a form written again for another number of items or changed by a cut the form that it came from; the values that the words before its tail said; the values that the first sentence said where it is a later sentence; how usual its rarest run is and how many of its runs are rare; and the first lesson with it");

impl SentenceForm {
    pub fn fixed_of(&self, slot: u32) -> Option<&Vec<u32>> {
        self.fixed.iter().find(|(seen, _)| *seen == slot).map(|(_, values)| values)
    }

    pub fn tied_of(&self, slot: u32) -> Option<u32> {
        self.tied.iter().find(|(seen, _)| *seen == slot).map(|(_, value)| *value)
    }

    pub fn item_at(&self, at: usize) -> Option<u32> {
        self.which.get(at).copied().filter(|number| *number != NO_NUMBER)
    }

    pub fn slot_at(&self, at: usize) -> Option<u32> {
        self.places.iter().find(|(place, _)| *place as usize == at).map(|(_, slot)| *slot)
    }

    pub fn is_place(&self, at: usize) -> bool {
        self.places.iter().any(|(place, _)| *place as usize == at)
    }

    pub fn tied_mask(&self) -> u64 {
        self.tied.iter().fold(0, |mask, (slot, _)| mask | slot_bit(*slot))
    }

    pub fn bound_to(&self) -> BoundKey {
        let mut key: BoundKey = self.fixed.clone();
        for (slot, value) in &self.tied {
            match key.iter_mut().find(|(seen, _)| seen == slot) {
                Some((_, values)) => values.push(*value),
                None => key.push((*slot, vec![*value])),
            }
        }
        canonical_bound(key)
    }

    pub fn phrase_spans(&self) -> Vec<(u32, (u32, u32))> {
        self.said_at
            .iter()
            .filter(|(_, places)| !places.is_empty())
            .map(|(slot, places)| (*slot, (places.iter().map(|(from, _)| *from).min().unwrap_or(0), places.iter().map(|(_, to)| *to).max().unwrap_or(0))))
            .collect()
    }

    pub fn side(&self, cut: u32) -> Option<(u64, u64)> {
        let mut head = 0u64;
        let mut tail = 0u64;
        for (slot, places) in &self.said_at {
            if places.iter().all(|(_, to)| *to <= cut) {
                head |= slot_bit(*slot);
            } else if places.iter().all(|(from, _)| *from >= cut) {
                tail |= slot_bit(*slot);
            } else {
                return None;
            }
        }
        Some((head, tail))
    }

    pub fn list_shape(&self, slot: u32) -> Option<(Vec<usize>, Vec<Vec<u32>>)> {
        let mut ats: Vec<usize> = self.places.iter().filter(|(at, seen)| *seen == slot && self.item_at(*at as usize).is_some()).map(|(at, _)| *at as usize).collect();
        ats.sort();
        let mut numbers: Vec<u32> = ats.iter().filter_map(|at| self.item_at(*at)).collect();
        numbers.sort();
        numbers.dedup();
        if ats.is_empty() || self.fixed_of(slot).is_some() || numbers.len() != ats.len() {
            return None;
        }
        let gaps: Vec<Vec<u32>> = ats.windows(WRITE_HEAD_AND_NEXT).map(|pair| self.key[pair[0] + 1..pair[1]].to_vec()).collect();
        for (gap, pair) in gaps.iter().zip(ats.windows(WRITE_HEAD_AND_NEXT)) {
            if gap.len() > WRITE_LIST_GAP || (pair[0] + 1..pair[1]).any(|at| self.is_place(at)) {
                return None;
            }
        }
        if gaps.len() > WRITE_HEAD_AND_NEXT {
            let inner = &gaps[..gaps.len() - 1];
            if inner.iter().any(|gap| *gap != inner[0]) {
                return None;
            }
        }
        Some((ats, gaps))
    }
}

pub const WRITE_HEAD_AND_NEXT: usize = 2;
because!(WRITE_HEAD_AND_NEXT, TemplateForms, "the units in a window that holds a unit and its neighbour: a place of a list is looked at with the place after it, and a form is counted with a mark before it and a mark after it");

pub fn canonical_bound(mut key: BoundKey) -> BoundKey {
    for (_, values) in key.iter_mut() {
        values.sort();
        values.dedup();
    }
    key.sort();
    key
}
because!(canonical_bound, TemplateForms, "a key of bound values in one fixed order, so that the order in which the lessons gave the values does not make two keys of one");

#[derive(Clone, Debug, Default)]
pub struct MadeForm {
    pub key: Vec<u32>,
    pub places: Vec<(u32, u32)>,
    pub slots: u64,
    pub fixed: Vec<(u32, Vec<(u32, u32)>)>,
    pub said_at: Vec<(u32, Vec<(u32, u32)>)>,
    pub which: Vec<u32>,
}
because!(MadeForm, TemplateForms, "what one lesson sentence gives when the reader agrees with its facts: the units of its form, its places, the slots that it says, where fixed words say a slot, where each slot is said, and the item number of each place of a list");

#[derive(Clone, Debug, Default)]
pub struct SlotPhrase {
    pub count: i64,
    pub places: Vec<u32>,
    pub values: Vec<u32>,
    pub beside: (u32, u32),
}
because!(SlotPhrase, TemplateForms, "a run of units that says one slot, between one unit to its left and one to its right: the lesson sentences that have it there, the places inside it, the values of the lessons whose fixed words it holds, and how many units beside the words of the slot it takes on each side");

#[derive(Clone, Debug, Default)]
pub struct NearLesson {
    pub lesson: u32,
    pub toks: Vec<u32>,
    pub values: Vec<u32>,
}
because!(NearLesson, TemplateForms, "a lesson for the choice among texts: its number, its units from the start mark to the end mark, and the value of each slot, with the mark of no number for a slot that the lesson does not have, so that two lists are compared slot by slot with no search");

pub type ListSides = (u32, u32);
because!(ListSides, TemplateForms, "the unit to the left and the unit to the right of a list or of a phrase; the mark of no number stands for any unit");

pub type ListWay = (Vec<u32>, Vec<u32>);
because!(ListWay, TemplateForms, "a way to write a list: the words between its items and the words before its last item; for a list of a pair of items the first part is empty");

#[derive(Clone, Debug, Default)]
pub struct FormStore {
    pub forms: Vec<SentenceForm>,
    pub lesson_forms: usize,
    pub form_at: NumberTable<(u32, Vec<u32>), u32>,
    pub signs: FirstSightTable<Vec<(u32, u32)>, ()>,
    pub clean: i64,
    pub left: NumberTable<(u32, u32, u32), FirstSightCount<u32>>,
    pub right: NumberTable<(u32, u32, u32), FirstSightCount<u32>>,
    pub right_units: NumberTable<(u32, u32), Vec<u32>>,
    pub in_place: NumberTable<(u32, u32), FirstSightCount<u32>>,
    pub lead: NumberTable<(u32, u32), FirstSightTable<u32, FirstSightCount<u32>>>,
    pub again: u64,
    pub starts: [NumberBag<Vec<u32>>; WRITE_HEAD_AND_NEXT],
    pub usual_runs: NumberTable<Vec<u32>, i64>,
    pub goes_on: NumberTable<u32, FirstSightCount<(u32, u32)>>,
    pub second: NumberTable<u32, FirstSightCount<u32>>,
    pub comes_after: NumberTable<u32, FirstSightCount<u32>>,
    pub next_units: NumberTable<u32, Vec<u32>>,
    pub start_end: NumberBag<(Vec<u32>, Vec<u32>)>,
    pub join_least: i64,
    pub join_pairs: i64,
    pub values_with: Vec<NumberTable<u32, FirstSightCount<u32>>>,
    pub list_gaps: NumberTable<Vec<u32>, i64>,
    pub list_two: FirstSightTable<u32, FirstSightTable<ListWay, FirstSightCount<ListSides>>>,
    pub list_more: FirstSightTable<u32, FirstSightTable<ListWay, FirstSightCount<ListSides>>>,
    pub list_pair: NumberTable<Vec<u32>, i64>,
    pub list_last: FirstSightTable<Vec<u32>, i64>,
    pub runs_seen: NumberBag<Vec<u32>>,
    pub between: FirstSightTable<ListSides, FirstSightTable<(u32, Vec<u32>), SlotPhrase>>,
    pub pairs_after: NumberTable<(Vec<u32>, u32), NumberBag<Vec<u32>>>,
    pub by_slots: FirstSightTable<u64, FirstSightTable<BoundKey, Vec<u32>>>,
    pub near: FirstSightTable<u64, Vec<NearLesson>>,
}
because!(FormStore, TemplateForms, CleanLessons, UsualRuns, JoinedLessonForms, ListWords, FormGroups, "the forms of sentence that the lessons give, and every count that says where a value fits. A lesson is clean when the reader reads its text back to exactly its facts; only clean lessons give forms. Counted for each place: the values beside the unit to the left and to the right, apart by the position of the place in a list. Joined forms: the head of one lesson sentence and the tail of another, when the units around the cut agree and every run over the cut stands in a lesson form. The ways in which the lessons write lists, the phrases that say one slot, the forms grouped by the slots that they say and the values that they are bound to, and the lessons grouped by their slots for the choice among texts. learn makes all of it from the lessons and the reader; the forms of the lessons come first in the run of forms and the joined forms after them");

impl FormStore {
    pub fn form(&self, number: u32) -> &SentenceForm {
        &self.forms[number as usize]
    }

    pub fn padded(lessons: &WriterLessons, key: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(key.len() + WRITE_HEAD_AND_NEXT);
        out.push(lessons.start);
        out.extend_from_slice(key);
        out.push(lessons.end);
        out
    }

    pub fn rarest(&self, lessons: &WriterLessons, key: &[u32]) -> i64 {
        let padded = FormStore::padded(lessons, key);
        let last = padded.len().saturating_sub(WRITE_USUAL_RUN).max(0);
        (0..=last).map(|at| self.usual_runs.get(&padded[at..(at + WRITE_USUAL_RUN).min(padded.len())]).copied().unwrap_or(0)).min().unwrap_or(0)
    }

    pub fn rare_runs(&self, lessons: &WriterLessons, key: &[u32]) -> i64 {
        let padded = FormStore::padded(lessons, key);
        let last = padded.len().saturating_sub(WRITE_USUAL_RUN).max(0);
        (0..=last).filter(|at| self.usual_runs.get(&padded[*at..(*at + WRITE_USUAL_RUN).min(padded.len())]).copied().unwrap_or(0) < WRITE_USUAL_SENTENCES).count() as i64
    }

    pub fn shared(one: &FirstSightCount<u32>, other: &FirstSightCount<u32>) -> f64 {
        let all_one = all_counts(one);
        let all_other = all_counts(other);
        if all_one == 0 || all_other == 0 {
            return f64::default();
        }
        compensated(one.rows().iter().filter(|(key, _)| other.holds(key)).map(|(key, count)| ((*count as f64) / (all_one as f64)).min((key_count(other, key) as f64) / (all_other as f64))))
    }

    pub fn alike(&self, one: u32, other: u32) -> f64 {
        let (Some(a), Some(b)) = (self.goes_on.get(&one), self.goes_on.get(&other)) else {
            return f64::default();
        };
        let all_a = all_counts(a);
        let all_b = all_counts(b);
        if all_a == 0 || all_b == 0 {
            return f64::default();
        }
        compensated(a.rows().iter().filter(|(way, _)| b.holds(way)).map(|(way, count)| ((*count as f64) / (all_a as f64)).min((key_count(b, way) as f64) / (all_b as f64))))
    }

    pub fn alike_before(&self, one: u32, other: u32) -> f64 {
        match (self.comes_after.get(&one), self.comes_after.get(&other)) {
            (Some(a), Some(b)) => FormStore::shared(a, b),
            _ => f64::default(),
        }
    }

    pub fn start_and_end(key: &[u32]) -> (Vec<u32>, Vec<u32>) {
        (key.iter().take(WRITE_OPEN_UNITS).copied().collect(), key[key.len().saturating_sub(WRITE_SAME_UNITS + 1)..].to_vec())
    }

    pub fn learn(lessons: &WriterLessons, reader: &TextReader) -> FormStore {
        let mut forms = FormStore::default();
        forms.learn_forms(lessons, reader);
        forms.learn_values_with(lessons);
        forms.learn_runs(lessons);
        forms.learn_joins(lessons, reader);
        forms.learn_list_words(lessons);
        forms.learn_phrases(lessons, reader);
        forms.group_forms(lessons);
        forms.learn_near(lessons);
        forms
    }
}
