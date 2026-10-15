use crate::base::keys::NO_NUMBER;
use crate::write::closing::ClosedLists;
use crate::write::endings::NameEndings;
use crate::write::fill::FilledPlaces;
use crate::write::forms::{FormStore, SentenceForm, WRITE_ALIKE_SHARE};
use crate::write::lessons::{FreshNumbers, WRITE_EMPTY_VALUE, WriterLessons};
use crate::write::numbers::{poisson_at_most, to_power};
use crate::write::reader::TextReader;
use crate::write::signs::first_low;
use crate::base::first_sight::{FirstSightCount, NumberTable, all_counts, key_count};
use crate::write::text::SurfaceText;
use patterns::{because, source};

pub struct PlaceFit;
source!(PlaceFit, "the fit of a value in a place of a template as a test on counts: the count of the value beside the unit to the left and to the right of the place is compared with what its share in all places of its slot predicts, by the law of small numbers of Poisson; a value is kept out only when its count is surely too low, so a value that the lessons never showed in a place can still go there");

pub const WRITE_UNFIT_CHANCE: f64 = 0.05;
because!(WRITE_UNFIT_CHANCE, PlaceFit, "a value is unfit for a place when its count there has this chance or less under the share that it has in all places of its slot");

pub const WRITE_FEW_SHARE: f64 = 0.7;
because!(WRITE_FEW_SHARE, PlaceFit, "a value is unfit for a place only when its count there is also under this share of what its share predicts: with many lessons a small lack is sure as well, and is no reason to keep a value out");

pub const WRITE_SIGNS_COMMON: f64 = 0.15;
because!(WRITE_SIGNS_COMMON, PlaceFit, "two units are forms of one word only when the first signs of the tokens after them have at most this share in common");

pub const WRITE_LEAD_LESSONS: i64 = 2;
because!(WRITE_LEAD_LESSONS, PlaceFit, "the lessons that must have another unit to the left of a value, for that unit to take the place of an unfit one");

pub const WRITE_REPEAT_LESSONS: i64 = 5;
because!(WRITE_REPEAT_LESSONS, PlaceFit, "the lessons for each value of a name slot, on the average, from which a word that stands with one value alone is taken as a word about that value");

#[derive(Clone, Debug, Default)]
pub struct ListClosing {
    pub table: NumberTable<u32, FirstSightCount<u32>>,
    pub open: Vec<u32>,
    pub top: u32,
    pub kinds: bool,
}
because!(ListClosing, PlaceFit, "how the lessons close the lists of one slot: for each unit after a list the items that stand last before it; the units that take every item; the unit that takes the most items; and whether the items of the slot are of more than one kind, so that the rule holds for the slot at all");

#[derive(Clone, Debug, Default)]
pub struct FilledForm {
    pub toks: Vec<u32>,
    pub key: Vec<u32>,
    pub items_at: Vec<(u32, u32)>,
}
because!(FilledForm, PlaceFit, "a form filled with the values of a list of facts: its tokens as the reader reads them, its units with the unit before a place changed or taken out where the value asked for it, and the item of a list in each place of a list");

pub struct PlaceFiller<'a> {
    pub lessons: &'a WriterLessons,
    pub reader: &'a TextReader,
    pub forms: &'a FormStore,
    pub fresh: FreshNumbers,
    pub name_forms: bool,
    pub(crate) closing: NumberTable<u32, ListClosing>,
    pub(crate) fit: NumberTable<(u32, u32, u32, u32, u32), (bool, bool)>,
    pub(crate) forms_of_one_word: NumberTable<(u32, u32), bool>,
    pub(crate) ends_main: NumberTable<(u32, u32), u32>,
    pub(crate) ends_fit: NumberTable<(u32, u32, u32), bool>,
}
because!(PlaceFiller, PlaceFit, ClosedLists, NameEndings, FilledPlaces, SurfaceText, "the tests that say whether a value fits a place, for one list of facts. A value does not go into a place when its count beside the unit to the left or to the right is surely below what its share in all places of the slot predicts; for an item of a list the counts are those of the places with the same position in a list. A value that never stood after a unit is tested by its first sign, when the unit has another form of the same word. A name is tested by the ending of its first word. A list is closed only by an item that the lessons show before the next unit. A place takes no value that a fixed word of the form speaks against. filled gives the form with the values in its places, with the unit before a place changed where the lessons have another form of that word for this value; text gives the filled form as people write it. Every answer is a function of the model and of what is asked, so what was found once is kept for the time of one text");

impl<'a> PlaceFiller<'a> {
    pub fn new(lessons: &'a WriterLessons, reader: &'a TextReader, forms: &'a FormStore) -> PlaceFiller<'a> {
        PlaceFiller { lessons, reader, forms, fresh: FreshNumbers::default(), name_forms: true, closing: NumberTable::default(), fit: NumberTable::default(), forms_of_one_word: NumberTable::default(), ends_main: NumberTable::default(), ends_fit: NumberTable::default() }
    }

    fn sum_of(seen: Option<&FirstSightCount<u32>>) -> i64 {
        seen.map_or(0, all_counts)
    }

    pub(crate) fn at(seen: Option<&FirstSightCount<u32>>, key: u32) -> i64 {
        seen.map_or(0, |seen| key_count(seen, &key))
    }

    pub(crate) fn rare(here: i64, mean: f64, bound: f64) -> bool {
        (here as f64) < WRITE_FEW_SHARE * mean && poisson_at_most(here, mean) < bound
    }

    pub fn fact_value_text(&self, value: u32) -> &str {
        self.lessons.fact_value_text(value, &self.fresh)
    }

    pub fn value_tokens(&self, value: u32) -> &[u32] {
        self.lessons.tokens_of_value(value, &self.fresh)
    }

    pub fn two_forms(&mut self, one: u32, other: u32) -> bool {
        if let Some(known) = self.forms_of_one_word.get(&(one, other)) {
            return *known;
        }
        let forms = self.forms;
        let empty: FirstSightCount<u32> = FirstSightCount::new();
        let second = |unit: u32| forms.second.get(&unit).unwrap_or(&empty);
        let signs = |unit: u32| self.lessons.surface.left_sign.get(&unit).unwrap_or(&empty);
        let answer = forms.alike_before(one, other) >= WRITE_ALIKE_SHARE && FormStore::shared(second(one), second(other)) >= WRITE_ALIKE_SHARE && FormStore::shared(signs(one), signs(other)) <= WRITE_SIGNS_COMMON;
        self.forms_of_one_word.insert((one, other), answer);
        answer
    }

    fn has_other_forms(&mut self, unit: u32) -> bool {
        let forms = self.forms;
        let Some(before) = forms.comes_after.get(&unit) else {
            return false;
        };
        let mut others: Vec<u32> = before.rows().iter().flat_map(|(earlier, _)| forms.next_units.get(earlier).map_or(&[][..], |next| next.as_slice()).iter().copied()).collect();
        others.sort();
        others.dedup();
        others.into_iter().any(|other| other != unit && self.two_forms(unit, other))
    }

    pub fn sign_fits(&mut self, unit: u32, value: u32) -> bool {
        let sign = self.lessons.syms.find(&first_low(self.fact_value_text(value))).unwrap_or(NO_NUMBER);
        let surface = &self.lessons.surface;
        let signs = surface.left_sign.get(&unit);
        let here = PlaceFiller::at(signs, sign);
        let mean = ((PlaceFiller::sum_of(signs) * key_count(&surface.signs, &sign)) as f64) / (all_counts(&surface.signs).max(1) as f64);
        !PlaceFiller::rare(here, mean, WRITE_UNFIT_CHANCE)
    }

    pub fn fit_at(&mut self, slot: u32, value: u32, left: u32, right: u32, position: u32) -> bool {
        let forms = self.forms;
        let share = forms.in_place.get(&(slot, position));
        let mut alone = 0;
        for (side, unit) in [(0u32, left), (1u32, right)] {
            let key = (side, slot, unit, value, position);
            let known = match self.fit.get(&key) {
                Some(known) => *known,
                None => {
                    let seen = if side == 0 { forms.left.get(&(slot, position, unit)) } else { forms.right.get(&(slot, position, unit)) };
                    let here = PlaceFiller::at(seen, value);
                    let mean = ((PlaceFiller::sum_of(seen) * PlaceFiller::at(share, value)) as f64) / (PlaceFiller::sum_of(share).max(1) as f64);
                    let mut fit = !PlaceFiller::rare(here, mean, WRITE_UNFIT_CHANCE);
                    let one = here == 0 && seen.is_some_and(|seen| seen.rows().iter().filter(|(_, count)| *count > 0).count() == 1);
                    if fit && side == 0 && here == 0 {
                        fit = self.sign_fits(unit, value) || !self.has_other_forms(unit);
                    }
                    self.fit.insert(key, (fit, one));
                    (fit, one)
                }
            };
            if !known.0 {
                return false;
            }
            alone += i32::from(known.1);
        }
        alone <= 1
    }

    pub fn other_lead(&mut self, slot: u32, value: u32, before: u32, unit: u32) -> u32 {
        let forms = self.forms;
        let lessons = self.lessons;
        let mut best: Option<(i64, u32)> = None;
        let better = |best: &Option<(i64, u32)>, many: i64, other: u32| best.is_none_or(|(count, top)| many > count || (many == count && lessons.rank[other as usize] > lessons.rank[top as usize]));
        if let Some(found) = forms.lead.get(&(slot, before)) {
            for (other, values) in found.rows() {
                let many = key_count(values, &value);
                if *other != unit && many >= WRITE_LEAD_LESSONS && self.two_forms(unit, *other) && better(&best, many, *other) {
                    best = Some((many, *other));
                }
            }
        }
        if best.is_none() {
            let sign = lessons.syms.find(&first_low(self.fact_value_text(value))).unwrap_or(NO_NUMBER);
            if let Some(found) = lessons.surface.lead_sign.get(&before) {
                for (other, signs) in found.rows() {
                    let many = key_count(signs, &sign);
                    if *other != unit && many >= WRITE_LEAD_LESSONS && self.two_forms(unit, *other) && better(&best, many, *other) {
                        best = Some((many, *other));
                    }
                }
            }
        }
        best.map_or(NO_NUMBER, |(_, other)| other)
    }

    fn seen_with(&self, slot: u32, unit: u32) -> Option<&FirstSightCount<u32>> {
        self.forms.values_with.get(slot as usize).and_then(|table| table.get(&unit))
    }

    pub fn says_carried(&self, form: &SentenceForm, unit: u32) -> bool {
        form.tied.iter().any(|(slot, value)| *value != WRITE_EMPTY_VALUE && !self.reader.carried_order.contains(slot) && self.seen_with(*slot, unit).map_or(0, |seen| seen.rows().len()) <= 1)
    }

    pub fn plain_of(&self, slot: u32) -> Option<u32> {
        self.reader.carried_order.contains(&slot).then(|| self.reader.plain[slot as usize])
    }

    pub fn one_value(&self, form: &SentenceForm, slot: u32, value: u32) -> bool {
        let reader = self.reader;
        let other_sort = !reader.carried_order.is_empty() && reader.carried_order.iter().any(|carried| form.tied_of(*carried) != Some(reader.plain[*carried as usize]));
        let Some(values) = self.lessons.values.get(slot as usize) else {
            return false;
        };
        if !other_sort || reader.plain_needs >> slot & 1 == 1 || all_counts(values) < WRITE_REPEAT_LESSONS * (values.rows().len() as i64) {
            return false;
        }
        form.key.iter().enumerate().any(|(at, unit)| !form.is_place(at) && self.seen_with(slot, *unit).is_some_and(|seen| seen.rows().len() == 1 && !seen.holds(&value)))
    }

    pub fn other_value(&self, form: &SentenceForm, slot: u32, value: u32) -> bool {
        let Some(values) = self.lessons.values.get(slot as usize) else {
            return false;
        };
        let every: i64 = values.rows().iter().filter(|(seen, _)| *seen != WRITE_EMPTY_VALUE).map(|(_, many)| *many).sum();
        form.key.iter().enumerate().any(|(at, unit)| {
            !form.is_place(at)
                && self.seen_with(slot, *unit).is_some_and(|seen| {
                    seen.rows().len() == 1 && !seen.holds(&value) && to_power((key_count(values, &seen.rows()[0].0) as f64) / (every as f64), all_counts(seen) as f64) < WRITE_UNFIT_CHANCE
                })
        })
    }
}
