use crate::base::keys::NO_NUMBER;
use crate::write::desk::WritingDesk;
use crate::write::forms::{BoundKey, ListWay, SentenceForm, WRITE_HEAD_AND_NEXT, WRITE_USUAL_SENTENCES};
use crate::write::lessons::slot_bit;
use crate::write::phrases::WRITE_LIST_MANY;
use crate::base::first_sight::{FirstSightTable, key_count};
use patterns::{because, source};
use std::sync::Arc;

pub struct OtherNumbers;
source!(OtherNumbers, "a form whose lesson had a list of some number of items serves a list of another number when it writes the list as one phrase: the words between the items are the words of the lesson again, or the words that the lessons use for a list of that number");

pub const WRITE_LIST_WAYS: usize = 3;
because!(WRITE_LIST_WAYS, OtherNumbers, "the ways to write a list of another number of items that are tried, where the lesson of the form does not show the words");

pub const WRITE_RESIZED_KEPT: usize = 12;
because!(WRITE_RESIZED_KEPT, OtherNumbers, "the forms written again for another number of items that are kept for one set of slots and bound values");

pub const WRITE_SIDE_KEYS: usize = 4;
because!(WRITE_SIDE_KEYS, OtherNumbers, "the counts by which a way to write a list is ranked: beside the same units on both sides, on the right alone, on the left alone, and anywhere");

impl<'a> WritingDesk<'a> {
    fn list_ways(&self, slot: u32, items: usize, left: u32, right: u32, last: Option<&[u32]>) -> Vec<ListWay> {
        let lessons = self.filler.lessons;
        let forms = self.filler.forms;
        let table = if items == WRITE_HEAD_AND_NEXT { &forms.list_two } else { &forms.list_more };
        let found = match table.get(&slot).filter(|found| !found.rows().is_empty()) {
            Some(found) => Some(found),
            None => table.get(&NO_NUMBER),
        };
        let Some(found) = found else {
            return Vec::new();
        };
        let mut ranked: Vec<(bool, [i64; WRITE_SIDE_KEYS], &ListWay)> = Vec::new();
        for (way, seen) in found.rows() {
            let end = &way.1;
            let like = match last {
                None => true,
                Some(last) => {
                    if items == WRITE_HEAD_AND_NEXT {
                        last.ends_with(end)
                    } else {
                        end.ends_with(last)
                    }
                }
            };
            ranked.push((like, [key_count(seen, &(left, right)), key_count(seen, &(NO_NUMBER, right)), key_count(seen, &(left, NO_NUMBER)), key_count(seen, &(NO_NUMBER, NO_NUMBER))], way));
        }
        ranked.sort_by(|one, other| other.0.cmp(&one.0).then(other.1.cmp(&one.1)).then_with(|| lessons.compare_units(&other.2.0, &one.2.0)).then_with(|| lessons.compare_units(&other.2.1, &one.2.1)));
        ranked.into_iter().take(WRITE_LIST_WAYS).map(|(_, _, way)| way.clone()).collect()
    }

    pub(crate) fn resized(&mut self, number: u32, numbers: &[(u32, u32)]) -> Vec<u32> {
        let lessons = self.filler.lessons;
        let forms = self.filler.forms;
        let form = self.form(number).clone();
        let mut wanted: Vec<(u32, u32)> = numbers.to_vec();
        wanted.sort_by(|one, other| lessons.slots.name(one.0).cmp(lessons.slots.name(other.0)));
        let mut choices: Vec<Vec<(usize, usize, Vec<(u32, u32, u32)>)>> = Vec::new();
        for (slot, many) in &wanted {
            let many = *many as usize;
            let Some((ats, gaps)) = form.list_shape(*slot) else {
                return Vec::new();
            };
            if gaps.iter().any(|gap| forms.list_gaps.get(gap).copied().unwrap_or(0) < WRITE_USUAL_SENTENCES) {
                return Vec::new();
            }
            let last_at = ats[ats.len() - 1];
            if many == 1 && forms.list_last.rows().iter().any(|(gap, count)| !gap.is_empty() && *count > forms.list_pair.get(gap).copied().unwrap_or(0) && form.key[last_at + 1..].starts_with(gap)) {
                return Vec::new();
            }
            let ways: Vec<ListWay> = if many == 1 {
                vec![(Vec::new(), Vec::new())]
            } else if ats.len() >= WRITE_LIST_MANY && many >= WRITE_LIST_MANY {
                vec![(gaps[0].clone(), gaps[gaps.len() - 1].clone())]
            } else {
                let left = if ats[0] > 0 { form.key[ats[0] - 1] } else { lessons.start };
                let right = if last_at + 1 < form.key.len() { form.key[last_at + 1] } else { lessons.end };
                self.list_ways(*slot, many, left, right, gaps.last().map(|gap| gap.as_slice()))
            };
            let mut phrases: Vec<(usize, usize, Vec<(u32, u32, u32)>)> = Vec::new();
            for (inner, end) in ways {
                let mut units: Vec<(u32, u32, u32)> = Vec::new();
                for item in 0..many {
                    if item > 0 {
                        let between = if item == many - 1 { &end } else { &inner };
                        units.extend(between.iter().map(|unit| (*unit, NO_NUMBER, NO_NUMBER)));
                    }
                    units.push((lessons.labels[*slot as usize], *slot, item as u32));
                }
                phrases.push((ats[0], last_at + 1, units));
            }
            choices.push(phrases);
        }
        let mut out: Vec<u32> = Vec::new();
        let mut pick = vec![0usize; choices.len()];
        if choices.iter().any(|phrases| phrases.is_empty()) {
            return out;
        }
        loop {
            let mut entries: Vec<(u32, u32, u32)> = form.key.iter().map(|unit| (*unit, NO_NUMBER, NO_NUMBER)).collect();
            for (at, slot) in &form.places {
                entries[*at as usize] = (form.key[*at as usize], *slot, form.which[*at as usize]);
            }
            let mut chosen: Vec<&(usize, usize, Vec<(u32, u32, u32)>)> = choices.iter().zip(&pick).map(|(phrases, at)| &phrases[*at]).collect();
            chosen.sort_by(|one, other| other.0.cmp(&one.0));
            for (first, last, units) in chosen {
                entries.splice(*first..*last, units.iter().copied());
            }
            let key: Vec<u32> = entries.iter().map(|entry| entry.0).collect();
            let places: Vec<(u32, u32)> = entries.iter().enumerate().filter(|(_, entry)| entry.1 != NO_NUMBER).map(|(at, entry)| (at as u32, entry.1)).collect();
            let mut said_at: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
            for (at, slot) in &places {
                match said_at.iter_mut().find(|(seen, _)| seen == slot) {
                    Some((_, seen)) => seen.push((*at, *at + 1)),
                    None => said_at.push((*slot, vec![(*at, *at + 1)])),
                }
            }
            let new = SentenceForm {
                usual: form.usual.min(forms.rarest(lessons, &key)),
                rare: forms.rare_runs(lessons, &key) + form.rare - forms.rare_runs(lessons, &form.key),
                which: entries.iter().map(|entry| entry.2).collect(),
                key,
                places,
                said_at,
                slots: form.slots,
                fixed: form.fixed.clone(),
                tied: form.tied.clone(),
                sign: form.sign,
                needs: form.needs.clone(),
                after: form.after.clone(),
                count: form.count,
                first: form.first,
                later: form.later,
                lesson: form.lesson,
                resized: Some((number, wanted.clone())),
                parts: None,
                edited: None,
            };
            out.push(self.keep(new));
            let mut turn = pick.len();
            loop {
                if turn == 0 {
                    return out;
                }
                turn -= 1;
                pick[turn] += 1;
                if pick[turn] < choices[turn].len() {
                    break;
                }
                pick[turn] = 0;
            }
        }
    }

    pub(crate) fn for_numbers(&mut self, slots: u64, fixed: &BoundKey, members: &[u32], numbers: &[(u32, u32)]) -> Vec<u32> {
        let lessons = self.filler.lessons;
        let mut mine: Vec<(u32, u32)> = numbers.iter().copied().filter(|(slot, _)| slots & slot_bit(*slot) != 0).collect();
        mine.sort();
        let cache_key = (slots, fixed.clone(), mine);
        if let Some(known) = self.resized_kept.get(&cache_key) {
            return known.clone();
        }
        if let Some(found) = self.share.and_then(|share| share.read().ok().and_then(|kept| kept.get(&cache_key).cloned())) {
            let order: Vec<u32> = found.iter().map(|form| self.keep(form.clone())).collect();
            self.resized_kept.insert(cache_key, order.clone());
            return order;
        }
        let have: Vec<Vec<u32>> = members.iter().map(|number| self.form(*number).key.clone()).collect();
        let mut made: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
        for number in members {
            let form = self.form(*number);
            let mut has: Vec<(u32, u32)> = Vec::new();
            for (at, slot) in &form.places {
                if form.item_at(*at as usize).is_some() {
                    match has.iter_mut().find(|(seen, _)| seen == slot) {
                        Some((_, many)) => *many += 1,
                        None => has.push((*slot, 1)),
                    }
                }
            }
            let change: Vec<(u32, u32)> = has.iter().filter_map(|(slot, many)| numbers.iter().find(|(seen, wanted)| seen == slot && wanted != many).copied()).collect();
            if change.is_empty() {
                continue;
            }
            for new in self.resized(*number, &change) {
                let key = self.form(new).key.clone();
                if have.contains(&key) {
                    continue;
                }
                match made.get(&key).copied() {
                    None => {
                        made.put(key, new);
                    }
                    Some(old) => {
                        let fresh = self.form(new).clone();
                        let known = self.filler.forms.forms.len();
                        let kept = &mut self.local[old as usize - known];
                        kept.count += fresh.count;
                        kept.first += fresh.first;
                        kept.later += fresh.later;
                        let mut both: Vec<(u32, Vec<u32>)> = Vec::new();
                        for (slot, values) in &kept.after {
                            if let Some((_, more)) = fresh.after.iter().find(|(seen, _)| seen == slot) {
                                let mut all = values.clone();
                                all.extend(more.iter().copied().filter(|value| !values.contains(value)));
                                both.push((*slot, all));
                            }
                        }
                        kept.after = both;
                    }
                }
            }
        }
        let mut order: Vec<u32> = made.rows().iter().map(|(_, number)| *number).collect();
        order.sort_by(|one, other| {
            let (a, b) = (self.form(*one), self.form(*other));
            a.rare.cmp(&b.rare).then(b.count.cmp(&a.count)).then_with(|| lessons.compare_units(&a.key, &b.key))
        });
        order.truncate(WRITE_RESIZED_KEPT);
        if let Some(share) = self.share {
            let made: Vec<SentenceForm> = order.iter().map(|number| self.form(*number).clone()).collect();
            if let Ok(mut kept) = share.write() {
                kept.entry(cache_key.clone()).or_insert_with(|| Arc::new(made));
            }
        }
        self.resized_kept.insert(cache_key, order.clone());
        order
    }
}
