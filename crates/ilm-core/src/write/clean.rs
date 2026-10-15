use crate::base::keys::NO_NUMBER;
use crate::write::forms::{FormStore, MadeForm, SentenceForm, WRITE_FIRST_ITEM, WRITE_HEAD_AND_NEXT, WRITE_LAST_ITEM, WRITE_ONLY_ITEM};
use crate::write::lessons::{GivenFacts, TextUnit, WRITE_EARLIER_UNIT, WRITE_EMPTY_VALUE, WriterLessons, slot_bit};
use crate::base::runs::run_place;
use crate::write::reader::TextReader;
use crate::base::first_sight::{FirstSightCount, all_counts};
use patterns::{because, source};

pub struct CleanLessons;
source!(CleanLessons, "a lesson is clean when the reader reads its text back to exactly its facts; only clean lessons give forms, so that no form holds words about a fact that its list of facts does not have");

pub const WRITE_AGAIN_SHARE: f64 = 0.05;
because!(WRITE_AGAIN_SHARE, CleanLessons, "a slot can be said in more than one sentence of a text when this share of its clean lessons does so");

pub const WRITE_START_SHARE: f64 = 0.03;
because!(WRITE_START_SHARE, CleanLessons, "a sentence starts in a usual way when its first units start this share of the lesson sentences at its position");

pub const WRITE_START_UNITS: usize = 2;
because!(WRITE_START_UNITS, CleanLessons, "the first units of a sentence that are counted as its start");

impl FormStore {
    pub(crate) fn form_of(lessons: &WriterLessons, reader: &TextReader, given: &GivenFacts, sent: &[TextUnit]) -> Option<MadeForm> {
        let toks: Vec<u32> = sent.iter().map(|unit| unit.sym).collect();
        if reader.other_name(lessons, &toks) {
            return None;
        }
        let mut slots = sent.iter().filter(|unit| unit.name()).fold(0u64, |mask, unit| mask | slot_bit(unit.slot));
        let mut placed: Vec<(u32, Vec<u32>)> = Vec::new();
        for unit in sent.iter().filter(|unit| unit.placed_item()) {
            match placed.iter_mut().find(|(slot, _)| *slot == unit.slot) {
                Some((_, items)) => {
                    if !items.contains(&unit.item) {
                        items.push(unit.item);
                    }
                }
                None => placed.push((unit.slot, vec![unit.item])),
            }
        }
        let mut spans: Vec<(u32, u32, u32)> = Vec::new();
        let mut fixed: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
        let mut items: Vec<(u32, Vec<(u32, Vec<(u32, u32)>)>)> = Vec::new();
        for found in reader.read(&toks) {
            let key = reader.keys[found.key as usize];
            if key.item != NO_NUMBER {
                if !given.items(key.slot).contains(&key.item) {
                    return None;
                }
                match items.iter_mut().find(|(slot, _)| *slot == key.slot) {
                    Some((_, seen)) => seen.push((key.item, found.spans)),
                    None => items.push((key.slot, vec![(key.item, found.spans)])),
                }
                continue;
            }
            let value = match given.value(key.slot) {
                Some(value) if TextReader::in_group(&found.group, value) => value,
                _ => return None,
            };
            slots |= slot_bit(key.slot);
            let tokens = &lessons.value_tokens[value as usize];
            if reader.as_it_is.get(tokens).is_some_and(|slots| slots.contains(&key.slot)) {
                let mut at = run_place(&toks, tokens, 0);
                while let Some(place) = at {
                    let end = place + tokens.len();
                    if found.spans.iter().any(|(from, to)| place < *to as usize && (*from as usize) < end) {
                        spans.push((place as u32, end as u32, key.slot));
                    }
                    at = run_place(&toks, tokens, place + 1);
                }
            }
            fixed.push((key.slot, found.spans));
        }
        let mut whole: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
        let mut lists: Vec<u32> = placed.iter().map(|(slot, _)| *slot).chain(items.iter().map(|(slot, _)| *slot)).collect();
        lists.sort_by(|one, other| lessons.slots.name(*one).cmp(lessons.slots.name(*other)));
        lists.dedup();
        for slot in lists {
            let mut found: Vec<u32> = placed.iter().filter(|(seen, _)| *seen == slot).flat_map(|(_, seen)| seen.iter().copied()).collect();
            let other = items.iter().find(|(seen, _)| *seen == slot);
            found.extend(other.iter().flat_map(|(_, seen)| seen.iter().map(|(item, _)| *item)));
            found.sort();
            found.dedup();
            let mut all: Vec<u32> = given.items(slot).to_vec();
            all.sort();
            all.dedup();
            if found.len() < all.len() {
                return None;
            }
            slots |= slot_bit(slot);
            if let Some((_, seen)) = other {
                if !seen.is_empty() {
                    whole.push((slot, seen.iter().flat_map(|(_, places)| places.iter().copied()).collect()));
                }
            }
        }
        spans.sort_by(|one, other| one.0.cmp(&other.0).then(other.1.cmp(&one.1)).then_with(|| lessons.slots.name(one.2).cmp(lessons.slots.name(other.2))));
        let mut starts: Vec<(u32, u32, u32)> = Vec::new();
        let mut from = 0u32;
        for (at, end, slot) in spans {
            if at >= from {
                match starts.iter_mut().find(|(seen, _, _)| *seen == at) {
                    Some(entry) => *entry = (at, end, slot),
                    None => starts.push((at, end, slot)),
                }
                from = end;
            } else if starts.iter().find(|(seen, _, _)| *seen == at).is_some_and(|(_, _, seen)| *seen != slot) {
                return None;
            }
        }
        let mut made = MadeForm { slots, ..MadeForm::default() };
        let mut in_key: Vec<u32> = Vec::with_capacity(sent.len());
        let mut at = 0usize;
        while at < sent.len() {
            let here = made.key.len() as u32;
            if let Some((_, end, slot)) = starts.iter().find(|(seen, _, _)| *seen as usize == at) {
                made.places.push((here, *slot));
                in_key.extend(std::iter::repeat_n(here, *end as usize - at));
                made.key.push(lessons.labels[*slot as usize]);
                made.which.push(NO_NUMBER);
                at = *end as usize;
                continue;
            }
            let unit = &sent[at];
            let mut number = NO_NUMBER;
            if unit.placed_item() {
                number = given.items(unit.slot).iter().position(|item| *item == unit.item).map_or(NO_NUMBER, |place| place as u32);
                made.places.push((here, unit.slot));
            } else if unit.name() {
                made.places.push((here, unit.slot));
            }
            in_key.push(here);
            made.key.push(unit.sym);
            made.which.push(number);
            at += 1;
        }
        for (at, slot) in &made.places {
            match made.said_at.iter_mut().find(|(seen, _)| seen == slot) {
                Some((_, places)) => places.push((*at, *at + 1)),
                None => made.said_at.push((*slot, vec![(*at, *at + 1)])),
            }
        }
        let placed_slots: Vec<u32> = made.said_at.iter().map(|(slot, _)| *slot).collect();
        made.fixed = fixed.into_iter().filter(|(slot, _)| !placed_slots.contains(slot)).collect();
        for (slot, places) in whole {
            match made.fixed.iter_mut().find(|(seen, _)| *seen == slot) {
                Some((_, seen)) => *seen = places,
                None => made.fixed.push((slot, places)),
            }
        }
        for (slot, places) in &made.fixed {
            let mapped = places.iter().map(|(from, to)| (in_key[*from as usize], in_key[*to as usize - 1] + 1));
            match made.said_at.iter_mut().find(|(seen, _)| seen == slot) {
                Some((_, seen)) => seen.extend(mapped),
                None => made.said_at.push((*slot, mapped.collect())),
            }
        }
        Some(made)
    }

    fn sign_of(&mut self, tied: &[(u32, u32)]) -> u32 {
        let mut key = tied.to_vec();
        key.sort();
        self.signs.row_of(key) as u32
    }

    pub(crate) fn learn_forms(&mut self, lessons: &WriterLessons, reader: &TextReader) {
        let slots = lessons.slots.len();
        let mut twice = vec![0i64; slots];
        let mut once = vec![0i64; slots];
        for (number, lesson) in lessons.rows.iter().enumerate() {
            let given = &lesson.given;
            let mut tied: Vec<(u32, u32)> = given.facts.iter().filter(|fact| fact.value == WRITE_EMPTY_VALUE || reader.carried[fact.slot as usize]).map(|fact| (fact.slot, fact.value)).collect();
            let made: Option<Vec<MadeForm>> = lesson.sents.iter().map(|(from, to)| FormStore::form_of(lessons, reader, given, &lesson.units[*from as usize..*to as usize])).collect();
            let Some(made) = made else {
                continue;
            };
            let said = made.iter().fold(0u64, |mask, one| mask | one.slots);
            let part: Vec<u32> = reader.carried_order.iter().map(|slot| given.value(*slot).unwrap_or(NO_NUMBER)).collect();
            let inside = reader.carried_in.get(&part).copied().unwrap_or(0);
            for slot in 0..slots as u32 {
                if inside & slot_bit(slot) != 0 && given.holds(slot) && said & slot_bit(slot) == 0 {
                    let value = given.value(slot).unwrap_or(WRITE_EMPTY_VALUE);
                    match tied.iter_mut().find(|(seen, _)| *seen == slot) {
                        Some(entry) => entry.1 = value,
                        None => tied.push((slot, value)),
                    }
                }
            }
            let tied_mask = tied.iter().fold(0u64, |mask, (slot, _)| mask | slot_bit(*slot));
            if said | tied_mask != given.mask() {
                continue;
            }
            self.clean += 1;
            for slot in 0..slots as u32 {
                let times = made.iter().filter(|one| one.slots & slot_bit(slot) != 0).count();
                if times > 0 {
                    once[slot as usize] += 1;
                    twice[slot as usize] += i64::from(times > 1);
                }
            }
            let sign = self.sign_of(&tied);
            let plain = reader.carried_order.iter().all(|slot| given.value(*slot) == Some(reader.plain[*slot as usize]));
            let said_first: Vec<(u32, u32)> = if plain {
                Vec::new()
            } else {
                (0..slots as u32).filter(|slot| made[0].slots & slot_bit(*slot) != 0 && !lessons.is_name(*slot) && !lessons.is_list(*slot)).map(|slot| (slot, given.value(slot).unwrap_or(WRITE_EMPTY_VALUE))).collect()
            };
            for (place, one) in made.iter().enumerate() {
                let at = match self.form_at.get(&(sign, one.key.clone())) {
                    Some(at) => *at as usize,
                    None => {
                        let at = self.forms.len();
                        self.form_at.insert((sign, one.key.clone()), at as u32);
                        self.forms.push(SentenceForm { key: one.key.clone(), places: one.places.clone(), slots: one.slots | tied_mask, said_at: one.said_at.clone(), lesson: (number as u32, NO_NUMBER), tied: tied.clone(), sign, which: one.which.clone(), ..SentenceForm::default() });
                        at
                    }
                };
                let form = &mut self.forms[at];
                for (slot, _) in &one.fixed {
                    let value = given.value(*slot).unwrap_or(WRITE_EMPTY_VALUE);
                    match form.fixed.iter_mut().find(|(seen, _)| seen == slot) {
                        Some((_, values)) => {
                            if !values.contains(&value) {
                                values.push(value);
                            }
                        }
                        None => form.fixed.push((*slot, vec![value])),
                    }
                }
                if place > 0 {
                    for (slot, value) in &said_first {
                        if one.slots & slot_bit(*slot) != 0 {
                            continue;
                        }
                        match form.after.iter_mut().find(|(seen, _)| seen == slot) {
                            Some((_, values)) => {
                                if !values.contains(value) {
                                    values.push(*value);
                                }
                            }
                            None => form.after.push((*slot, vec![*value])),
                        }
                    }
                }
                form.count += 1;
                if place == 0 {
                    form.first += 1;
                } else {
                    form.later += 1;
                }
                let key = &one.key;
                let ends = |slot: u32| -> (u32, u32) {
                    let mut mine = one.places.iter().filter(|(at, seen)| *seen == slot && one.which[*at as usize] != NO_NUMBER).map(|(at, _)| *at);
                    let first = mine.next().unwrap_or(NO_NUMBER);
                    (first, mine.last().unwrap_or(first))
                };
                for (at, slot) in &one.places {
                    if lessons.is_name(*slot) {
                        continue;
                    }
                    let at = *at as usize;
                    let number = one.which[at];
                    let (value, position) = if number == NO_NUMBER {
                        (given.value(*slot).unwrap_or(WRITE_EMPTY_VALUE), WRITE_ONLY_ITEM)
                    } else {
                        let (first, last) = ends(*slot);
                        (given.items(*slot)[number as usize], u32::from(at as u32 == first) * WRITE_FIRST_ITEM + u32::from(at as u32 == last) * WRITE_LAST_ITEM)
                    };
                    let before = if at > 0 { key[at - 1] } else { lessons.start };
                    *self.left.entry((*slot, position, before)).or_default().at(value) += 1;
                    if at > 0 && !one.places.iter().any(|(place, _)| *place as usize == at - 1) {
                        let earlier = if at > 1 { key[at - WRITE_EARLIER_UNIT] } else { lessons.start };
                        *self.lead.entry((*slot, earlier)).or_default().at(before).at(value) += 1;
                    }
                    let after = if at + 1 < key.len() { key[at + 1] } else { lessons.end };
                    let seen = self.right.entry((*slot, position, after)).or_default();
                    if seen.rows().is_empty() {
                        self.right_units.entry((*slot, position)).or_default().push(after);
                    }
                    *seen.at(value) += 1;
                    *self.in_place.entry((*slot, position)).or_default().at(value) += 1;
                }
            }
        }
        for slot in 0..slots as u32 {
            if once[slot as usize] > 0 && (twice[slot as usize] as f64) >= WRITE_AGAIN_SHARE * (once[slot as usize] as f64) {
                self.again |= slot_bit(slot);
            }
        }
        let mut starts: [FirstSightCount<Vec<u32>>; WRITE_HEAD_AND_NEXT] = [FirstSightCount::new(), FirstSightCount::new()];
        for form in &self.forms {
            let start: Vec<u32> = form.key.iter().take(WRITE_START_UNITS).copied().collect();
            *starts[0].at(start.clone()) += form.first;
            *starts[1].at(start) += form.later;
        }
        for (which, seen) in starts.iter().enumerate() {
            let all = all_counts(seen) as f64;
            self.starts[which] = seen.rows().iter().filter(|(_, count)| (*count as f64) >= WRITE_START_SHARE * all).map(|(start, _)| start.clone()).collect();
        }
        self.lesson_forms = self.forms.len();
    }
}
