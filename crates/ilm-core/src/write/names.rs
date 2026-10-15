use crate::write::find::WantedValue;
use crate::write::lessons::{WRITE_EMPTY_VALUE, WriterLessons};
use crate::write::signs::{list_cuts, low_tokens};
use crate::base::first_sight::{FirstSightCount, NumberTable, key_count};
use patterns::{because, source};

pub struct NameSlots;
source!(NameSlots, "a slot is a name slot when its value stands word for word in nearly all of its lesson texts, and a value of it is a sure name when nearly all lesson texts that hold it have it as a value; a slot holds lists when its values share items between cuts");

pub const WRITE_NAME_SHARE: f64 = 0.95;
because!(WRITE_NAME_SHARE, NameSlots, "a slot is a name slot when this share of its lesson texts holds its value word for word");

pub const WRITE_NAME_SURE: f64 = 0.9;
because!(WRITE_NAME_SURE, NameSlots, "a value of a name slot is a sure name when this share of the lesson texts that hold it have it as a value");

pub const WRITE_END_SHARE: f64 = 0.02;
because!(WRITE_END_SHARE, NameSlots, "a token ends a sentence when it ends this share of the lesson texts");

pub const WRITE_LIST_ITEMS_SHARE: f64 = 0.5;
because!(WRITE_LIST_ITEMS_SHARE, NameSlots, "a slot holds lists when this share of the items of its values with a cut stand in more than one value");

pub const WRITE_LIST_VALUES: usize = 2;
because!(WRITE_LIST_VALUES, NameSlots, "a slot holds lists when this many of its values hold the cut, and an item counts when it stands in this many values");

impl WriterLessons {
    pub(crate) fn learn_lists(&mut self) {
        let mut raw: Vec<Vec<u32>> = vec![Vec::new(); self.slots.len()];
        for lesson in &self.rows {
            for (slot, value) in &lesson.facts {
                if !raw[*slot as usize].contains(value) {
                    raw[*slot as usize].push(*value);
                }
            }
        }
        self.cuts = vec![None; self.slots.len()];
        for (slot, values) in raw.iter().enumerate() {
            let texts: Vec<&str> = values.iter().map(|value| self.vals.name(*value)).collect();
            let mut cuts: Vec<String> = texts.iter().flat_map(|text| list_cuts(text)).collect();
            cuts.sort();
            cuts.dedup();
            let mut best: Option<(usize, String)> = None;
            for cut in cuts {
                let lists: Vec<&str> = texts.iter().copied().filter(|text| text.contains(cut.as_str())).collect();
                let mut inside: FirstSightCount<&str> = FirstSightCount::new();
                for text in &texts {
                    let mut once: Vec<&str> = Vec::new();
                    for item in text.split(cut.as_str()) {
                        if !once.contains(&item) {
                            once.push(item);
                            *inside.at(item) += 1;
                        }
                    }
                }
                let mut items: Vec<&str> = lists.iter().flat_map(|text| text.split(cut.as_str())).collect();
                items.sort();
                items.dedup();
                let again = items.iter().filter(|item| key_count(&inside, item) >= WRITE_LIST_VALUES as i64).count();
                if lists.len() >= WRITE_LIST_VALUES && (again as f64) >= WRITE_LIST_ITEMS_SHARE * (items.len() as f64) && best.as_ref().is_none_or(|(most, _)| lists.len() > *most) {
                    best = Some((lists.len(), cut));
                }
            }
            self.cuts[slot] = best.map(|(_, cut)| cut);
        }
    }

    pub(crate) fn wanted<'a>(&self, cache: &'a mut NumberTable<u32, WantedValue>, value: u32) -> &'a WantedValue {
        cache.entry(value).or_insert_with(|| WantedValue::new(self.vals.name(value)))
    }

    pub(crate) fn learn_names(&mut self, cache: &mut NumberTable<u32, WantedValue>) {
        let slots = self.slots.len();
        let mut have = vec![0i64; slots];
        let mut whole = vec![0i64; slots];
        let mut values: Vec<FirstSightCount<u32>> = vec![FirstSightCount::new(); slots];
        let mut last: FirstSightCount<u32> = FirstSightCount::new();
        let mut texts: Vec<Vec<u32>> = Vec::with_capacity(self.rows.len());
        for at in 0..self.rows.len() {
            for (slot, value) in self.rows[at].facts.clone() {
                *values[slot as usize].at(value) += 1;
                if value != WRITE_EMPTY_VALUE {
                    have[slot as usize] += 1;
                    whole[slot as usize] += i64::from(self.wanted(cache, value).stands(&self.rows[at].text));
                }
            }
            let tokens: Vec<u32> = low_tokens(&self.rows[at].text).iter().map(|token| self.syms.number(token)).collect();
            if let Some(end) = tokens.last() {
                *last.at(*end) += 1;
            }
            texts.push(tokens);
        }
        self.name_slot = (0..slots).map(|slot| have[slot] > 0 && self.cuts[slot].is_none() && (whole[slot] as f64) >= WRITE_NAME_SHARE * (have[slot] as f64)).collect();
        let lessons = self.rows.len() as f64;
        self.ends = last.rows().iter().filter(|(_, count)| (*count as f64) >= WRITE_END_SHARE * lessons).map(|(token, _)| *token).collect();
        self.value_tokens = Vec::with_capacity(self.vals.len());
        for value in 0..self.vals.len() {
            let tokens: Vec<u32> = low_tokens(self.vals.name(value as u32)).iter().map(|token| self.syms.number(token)).collect();
            self.value_tokens.push(tokens);
        }
        let mut names: NumberTable<u32, Vec<Vec<u32>>> = NumberTable::default();
        for slot in 0..slots {
            if !self.name_slot[slot] {
                continue;
            }
            for (value, _) in values[slot].rows() {
                let tokens = &self.value_tokens[*value as usize];
                if let Some(first) = tokens.first() {
                    let group = names.entry(*first).or_default();
                    if !group.contains(tokens) {
                        group.push(tokens.clone());
                    }
                }
            }
        }
        let mut held: FirstSightCount<Vec<u32>> = FirstSightCount::new();
        let mut given: FirstSightCount<Vec<u32>> = FirstSightCount::new();
        for (lesson, tokens) in self.rows.iter().zip(&texts) {
            let own: Vec<&Vec<u32>> = lesson.facts.iter().filter(|(slot, _)| self.name_slot[*slot as usize]).map(|(_, value)| &self.value_tokens[*value as usize]).collect();
            let mut found: Vec<&Vec<u32>> = Vec::new();
            for (at, token) in tokens.iter().enumerate() {
                for name in names.get(token).map_or(&[][..], |group| group.as_slice()) {
                    if tokens[at..].starts_with(name) && !found.contains(&name) {
                        found.push(name);
                    }
                }
            }
            for name in found {
                *held.at(name.clone()) += 1;
                if own.contains(&name) {
                    *given.at(name.clone()) += 1;
                }
            }
        }
        self.sure_names = NumberTable::default();
        for group in names.values() {
            for name in group {
                let seen = key_count(&held, name);
                if seen > 0 && (key_count(&given, name) as f64) >= WRITE_NAME_SURE * (seen as f64) {
                    self.sure_names.entry(name[0]).or_default().push(name.clone());
                }
            }
        }
        self.values = values;
    }
}
