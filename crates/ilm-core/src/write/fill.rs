use crate::base::keys::NO_NUMBER;
use crate::base::sums::compensated;
use crate::write::fit::{FilledForm, PlaceFiller, WRITE_LEAD_LESSONS};
use crate::write::forms::{SentenceForm, WRITE_FIRST_ITEM, WRITE_LAST_ITEM, WRITE_ONLY_ITEM};
use crate::write::lessons::{GivenFacts, WRITE_EARLIER_UNIT, WRITE_EMPTY_VALUE};
use crate::write::numbers::natural_log;
use crate::write::signs::letter_or_digit;
use crate::base::first_sight::key_count;
use patterns::{because, source};

pub struct FilledPlaces;
source!(FilledPlaces, "a template is filled value by value, each tested for its fit; the items of a list take the order in which each stands most often beside the units of its place, and the unit before a place changes to another form of the same word when the value asks for it");

pub const WRITE_PERMUTE_MOST: usize = 4;
because!(WRITE_PERMUTE_MOST, FilledPlaces, "the most items of a list whose order in a form is chosen by trying every order; with more, the first and the last are chosen by their counts and the others stay as given");

pub const WRITE_NOT_CLOSED: f64 = 1000.0;
because!(WRITE_NOT_CLOSED, FilledPlaces, "what an order of items loses when the lessons do not close a list with its last item before the next unit: more than every sum of logarithms of counts can give");

pub const WRITE_PAIR_OF_UNITS: usize = 2;
because!(WRITE_PAIR_OF_UNITS, FilledPlaces, "units side by side at a seam: the test for a token that would stand twice in a row where a place meets its neighbour");

pub fn next_order(order: &mut [usize]) -> bool {
    let Some(turn) = (1..order.len()).rev().find(|at| order[*at - 1] < order[*at]) else {
        return false;
    };
    let swap = (turn..order.len()).rev().find(|at| order[*at] > order[turn - 1]).unwrap_or(turn);
    order.swap(turn - 1, swap);
    order[turn..].reverse();
    true
}
because!(next_order, FilledPlaces, "the next order of the places of the items of a list, in the order of a dictionary over the places; of two orders with the same counts the one that comes first in this order is kept, so the walk must go in this order");

impl<'a> PlaceFiller<'a> {
    pub fn filled(&mut self, form: &SentenceForm, given: &GivenFacts, loose: u32) -> Option<FilledForm> {
        let lessons = self.lessons;
        for (slot, values) in &form.fixed {
            if !given.value(*slot).is_some_and(|value| values.contains(&value)) {
                return None;
            }
        }
        for (slot, value) in &form.tied {
            if given.value(*slot) == Some(*value) {
                continue;
            }
            let allowed = loose > 0 && given.has(*slot) && if loose == 1 { *value == self.plain_of(*slot).unwrap_or(WRITE_EMPTY_VALUE) } else { self.plain_of(*slot).is_some() };
            if !allowed {
                return None;
            }
        }
        let mut key: Vec<u32> = form.key.clone();
        let mut items_at: Vec<(u32, u32)> = Vec::new();
        let mut lists: Vec<(u32, Vec<u32>)> = Vec::new();
        for (at, slot) in &form.places {
            if form.item_at(*at as usize).is_some() {
                match lists.iter_mut().find(|(seen, _)| seen == slot) {
                    Some((_, ats)) => ats.push(*at),
                    None => lists.push((*slot, vec![*at])),
                }
            }
        }
        let ends = |slot: u32| -> (u32, u32) { lists.iter().find(|(seen, _)| *seen == slot).map_or((NO_NUMBER, NO_NUMBER), |(_, ats)| (ats[0], ats[ats.len() - 1])) };
        let position = |at: u32, slot: u32| -> u32 {
            if form.item_at(at as usize).is_none() {
                return WRITE_ONLY_ITEM;
            }
            let (first, last) = ends(slot);
            u32::from(at == first) * WRITE_FIRST_ITEM + u32::from(at == last) * WRITE_LAST_ITEM
        };
        for (slot, ats) in &lists {
            let fact = given.fact(*slot).filter(|fact| fact.list)?;
            let value = &fact.items;
            let mut numbers: Vec<u32> = ats.iter().filter_map(|at| form.item_at(*at as usize)).collect();
            numbers.sort();
            numbers.dedup();
            if form.fixed_of(*slot).is_some() || numbers.len() != ats.len() {
                for at in ats {
                    let number = form.item_at(*at as usize).unwrap_or(0) as usize;
                    items_at.push((*at, *value.get(number)?));
                }
                continue;
            }
            if value.len() != ats.len() {
                return None;
            }
            let last_place = ats[ats.len() - 1];
            let beside = |filler: &mut PlaceFiller<'a>, at: u32, item: u32| -> f64 {
                let here = at as usize;
                let left = if here > 0 { form.key[here - 1] } else { lessons.start };
                let right = if here + 1 < form.key.len() { form.key[here + 1] } else { lessons.end };
                let place = position(at, *slot);
                let before = PlaceFiller::at(filler.forms.left.get(&(*slot, place, left)), item);
                let after = PlaceFiller::at(filler.forms.right.get(&(*slot, place, right)), item);
                let open = at == last_place && !filler.closes(*slot, item, right);
                natural_log((1 + before) as f64) + natural_log((1 + after) as f64) - WRITE_NOT_CLOSED * f64::from(open)
            };
            let chosen: Vec<u32> = if ats.len() > WRITE_PERMUTE_MOST {
                let mut rest: Vec<u32> = value.clone();
                let pick = |filler: &mut PlaceFiller<'a>, rest: &mut Vec<u32>, at: u32| -> u32 {
                    let mut best: Option<(f64, usize)> = None;
                    for (place, item) in rest.iter().enumerate() {
                        let first = rest.iter().position(|seen| seen == item).unwrap_or(place);
                        let score = beside(filler, at, *item);
                        if best.is_none_or(|(top, top_place)| score > top || (score == top && first < top_place)) {
                            best = Some((score, first));
                        }
                    }
                    let place = best.map_or(0, |(_, place)| place);
                    rest.remove(place)
                };
                let first = pick(self, &mut rest, ats[0]);
                let last = pick(self, &mut rest, ats[ats.len() - 1]);
                let mut order = vec![first];
                order.extend(rest);
                order.push(last);
                order
            } else {
                let mut best: Option<(f64, Vec<u32>)> = None;
                let mut order: Vec<usize> = (0..value.len()).collect();
                let mut scores: Vec<f64> = Vec::with_capacity(ats.len());
                loop {
                    scores.clear();
                    for (at, place) in ats.iter().zip(&order) {
                        scores.push(beside(self, *at, value[*place]));
                    }
                    let count = compensated(scores.iter().copied());
                    if best.as_ref().is_none_or(|(top, _)| count > *top) {
                        best = Some((count, order.iter().map(|place| value[*place]).collect()));
                    }
                    if !next_order(&mut order) {
                        break;
                    }
                }
                best.map(|(_, order)| order).unwrap_or_default()
            };
            items_at.extend(ats.iter().copied().zip(chosen));
        }
        let mut written: Vec<(usize, u32, bool)> = Vec::with_capacity(form.places.len());
        for (at, slot) in &form.places {
            let here = *at as usize;
            let value = match items_at.iter().find(|(place, _)| place == at) {
                Some((_, item)) => *item,
                None => given.fact(*slot).filter(|fact| !fact.list && fact.value != WRITE_EMPTY_VALUE)?.value,
            };
            if self.fact_value_text(value).is_empty() {
                return None;
            }
            let mut left = if here > 0 { key[here - 1] } else { lessons.start };
            let right = if here + 1 < key.len() { key[here + 1] } else { lessons.end };
            let earlier = if here > 1 { key[here - WRITE_EARLIER_UNIT] } else { lessons.start };
            let left_free = here > 0 && !form.is_place(here - 1);
            if lessons.is_name(*slot) {
                if self.name_forms && !self.end_fits(*slot, left, value) {
                    let main = self.end_main(*slot, value);
                    if main == NO_NUMBER || !left_free || !self.end_fits(*slot, main, value) {
                        return None;
                    }
                    if earlier == main {
                        key[here - 1] = lessons.blank;
                    } else if key_count(&lessons.surface.lead_end[*slot as usize], &(earlier, main, self.ending_of(value))) >= WRITE_LEAD_LESSONS {
                        key[here - 1] = main;
                        left = main;
                    } else {
                        return None;
                    }
                }
                if left_free && key[here - 1] != lessons.blank && !self.sign_fits(left, value) {
                    let other = self.other_lead(*slot, value, earlier, left);
                    if other != NO_NUMBER {
                        key[here - 1] = other;
                    }
                }
                if self.one_value(form, *slot, value) {
                    return None;
                }
                written.push((here, value, false));
                continue;
            }
            let listed = form.item_at(here).is_some();
            if !listed && self.other_value(form, *slot, value) {
                return None;
            }
            if listed && *at == ends(*slot).1 && !self.closes(*slot, value, right) {
                return None;
            }
            let place = position(*at, *slot);
            if !self.fit_at(*slot, value, left, right, place) {
                let other = if left_free { self.other_lead(*slot, value, earlier, left) } else { NO_NUMBER };
                if other == NO_NUMBER || !self.fit_at(*slot, value, other, right, place) {
                    return None;
                }
                key[here - 1] = other;
            }
            written.push((here, value, !listed));
        }
        let tokens_at = |here: usize| -> Option<(&[u32], bool)> { written.iter().find(|(at, _, _)| *at == here).map(|(_, value, shown)| (self.value_tokens(*value), *shown)) };
        if lessons.surface.no_double {
            for here in 1..key.len() {
                let last = tokens_at(here - 1).map_or(Some(key[here - 1]), |(tokens, _)| tokens.last().copied());
                let first = tokens_at(here).map_or(Some(key[here]), |(tokens, _)| tokens.first().copied());
                if let (Some(last), Some(first)) = (last, first) {
                    if last == first && letter_or_digit(lessons.unit_text(first, &self.fresh)) {
                        return None;
                    }
                }
            }
        }
        let mut toks: Vec<u32> = Vec::with_capacity(key.len() + written.len());
        for (here, unit) in key.iter().enumerate() {
            if *unit == lessons.blank {
                continue;
            }
            match tokens_at(here) {
                Some((tokens, true)) => toks.extend_from_slice(tokens),
                _ => toks.push(*unit),
            }
        }
        Some(FilledForm { toks, key, items_at })
    }

    pub fn in_places(form: &SentenceForm, items_at: &[(u32, u32)]) -> Vec<(u32, Vec<u32>)> {
        let mut out: Vec<(u32, Vec<u32>)> = Vec::new();
        for (at, slot) in &form.places {
            let Some((_, item)) = items_at.iter().find(|(place, _)| place == at) else {
                continue;
            };
            match out.iter_mut().find(|(seen, _)| seen == slot) {
                Some((_, items)) => {
                    if !items.contains(item) {
                        items.push(*item);
                    }
                }
                None => out.push((*slot, vec![*item])),
            }
        }
        out
    }
}
