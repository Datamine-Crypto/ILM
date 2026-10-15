use crate::base::keys::NO_NUMBER;
use crate::write::fit::{ListClosing, PlaceFiller, WRITE_UNFIT_CHANCE};
use crate::write::forms::{WRITE_ALIKE_SHARE, WRITE_LAST_ITEM, WRITE_ONLY_ITEM};
use crate::base::first_sight::{FirstSightCount, all_counts, key_count};
use patterns::{because, source};

pub struct ClosedLists;
source!(ClosedLists, "the last item of a list must be one that the lessons show before the next unit, so that a list does not end with no noun; a unit that takes most items of a slot takes every item");

pub const WRITE_OPEN_ITEMS_SHARE: f64 = 0.5;
because!(WRITE_OPEN_ITEMS_SHARE, ClosedLists, "a unit takes every item of a list before it when the lessons have this share of the items of the slot there");

impl<'a> PlaceFiller<'a> {
    fn closing_of(&mut self, slot: u32) -> &ListClosing {
        if !self.closing.contains_key(&slot) {
            let forms = self.forms;
            let mut made = ListClosing { top: NO_NUMBER, ..ListClosing::default() };
            let mut count: FirstSightCount<u32> = FirstSightCount::new();
            let mut units: Vec<u32> = Vec::new();
            for position in [WRITE_LAST_ITEM, WRITE_ONLY_ITEM] {
                for other in forms.right_units.get(&(slot, position)).map_or(&[][..], |units| units.as_slice()) {
                    let Some(seen) = forms.right.get(&(slot, position, *other)) else {
                        continue;
                    };
                    if !units.contains(other) {
                        units.push(*other);
                    }
                    let all = made.table.entry(*other).or_default();
                    for (item, many) in seen.rows() {
                        *all.at(*item) += *many;
                    }
                }
                if let Some(seen) = forms.in_place.get(&(slot, position)) {
                    for (item, many) in seen.rows() {
                        *count.at(*item) += *many;
                    }
                }
            }
            let items = count.rows().len() as f64;
            let first: Vec<u32> = units.iter().copied().filter(|other| (made.table[other].rows().len() as f64) >= WRITE_OPEN_ITEMS_SHARE * items).collect();
            made.open = first.clone();
            for other in &units {
                if !made.open.contains(other) && first.iter().any(|one| forms.alike_before(*other, *one) >= WRITE_ALIKE_SHARE) {
                    made.open.push(*other);
                }
            }
            let mut top: Option<(usize, u32)> = None;
            for other in &units {
                let many = made.table[other].rows().len();
                let better = top.is_none_or(|(most, unit)| many > most || (many == most && self.lessons.rank[*other as usize] > self.lessons.rank[unit as usize]));
                if better {
                    top = Some((many, *other));
                }
            }
            made.top = top.map_or(NO_NUMBER, |(_, unit)| unit);
            let narrow: i64 = units.iter().filter(|other| !made.open.contains(other)).map(|other| all_counts(&made.table[other])).sum();
            let every = all_counts(&count);
            for (item, many) in count.rows() {
                let here: i64 = units.iter().filter(|other| !made.open.contains(other)).map(|other| key_count(&made.table[other], item)).sum();
                let mean = ((*many * narrow) as f64) / (every.max(1) as f64);
                if PlaceFiller::rare(here, mean, WRITE_UNFIT_CHANCE) {
                    made.kinds = true;
                }
            }
            self.closing.insert(slot, made);
        }
        &self.closing[&slot]
    }

    pub fn closes(&mut self, slot: u32, item: u32, unit: u32) -> bool {
        let last = self.value_tokens(item).last().copied().unwrap_or(NO_NUMBER);
        let closing = self.closing_of(slot);
        let Some(seen) = closing.table.get(&unit) else {
            return true;
        };
        if !closing.kinds || closing.open.contains(&unit) || seen.rows().is_empty() {
            return true;
        }
        key_count(seen, &item) > 0 || last == closing.top
    }
}
