use crate::base::keys::NO_NUMBER;
use crate::write::find::WantedValue;
use crate::write::lessons::{WRITE_EARLIER_UNIT, WRITE_EMPTY_VALUE, WriterLessons, WrittenSurface};
use crate::write::signs::{first_low, letter_or_digit, name_ending};
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberTable};
use patterns::{because, source};

pub struct WritingHabits;
source!(WritingHabits, "how the lessons write is counted as habits: the most frequent written form of a token, whether a space stands between neighbouring tokens, and the first signs and endings that follow a unit");

pub const WRITE_DOUBLE_SHARE: f64 = 0.01;
because!(WRITE_DOUBLE_SHARE, WritingHabits, "no token is written twice in a row when at most this share of the lesson texts does so");

pub const WRITE_ENDING_SIGNS: usize = 2;
because!(WRITE_ENDING_SIGNS, WritingHabits, "the last signs of the first word of a name that are counted with the unit to its left");

impl WriterLessons {
    pub(crate) fn learn_surface(&mut self, cache: &mut NumberTable<u32, WantedValue>) {
        let slots = self.slots.len();
        let mut written: FirstSightTable<(u32, u32), FirstSightCount<u32>> = FirstSightTable::new();
        let mut spaced: FirstSightTable<(u32, u32), (i64, i64)> = FirstSightTable::new();
        let mut spaced_one: FirstSightTable<u32, (i64, i64)> = FirstSightTable::new();
        let mut value_as: FirstSightTable<(u32, u32), FirstSightCount<u32>> = FirstSightTable::new();
        let mut surface = WrittenSurface { left_end: vec![FirstSightTable::new(); slots], lead_end: vec![FirstSightCount::new(); slots], ..WrittenSurface::default() };
        let mut doubled = 0i64;
        for at in 0..self.rows.len() {
            let lesson = self.rows[at].clone();
            let text = lesson.text.as_str();
            for (from, to) in &lesson.sents {
                let sent = &lesson.units[*from as usize..*to as usize];
                for (place, unit) in sent.iter().enumerate() {
                    let shown = &text[unit.start as usize..unit.end as usize];
                    let before = if place > 0 { sent[place - 1].sym } else { self.start };
                    if unit.name() {
                        let ending = self.syms.number(&name_ending(shown, WRITE_ENDING_SIGNS));
                        *surface.left_end[unit.slot as usize].at(before).at(ending) += 1;
                        if place > 0 {
                            let earlier = if place > 1 { sent[place - WRITE_EARLIER_UNIT].sym } else { self.start };
                            *surface.lead_end[unit.slot as usize].at((earlier, before, ending)) += 1;
                        }
                    }
                    if place == 0 {
                        continue;
                    }
                    if unit.plain() {
                        let shape = self.shapes.number(shown);
                        *written.at((NO_NUMBER, unit.sym)).at(shape) += 1;
                        *written.at((before, unit.sym)).at(shape) += 1;
                    }
                    doubled += i64::from(unit.sym == before && letter_or_digit(self.syms.name(unit.sym)));
                    let sign = self.syms.number(&first_low(shown));
                    *surface.left_sign.entry(before).or_default().at(sign) += 1;
                    *surface.signs.at(sign) += 1;
                    if sent[place - 1].plain() {
                        let earlier = if place > 1 { sent[place - WRITE_EARLIER_UNIT].sym } else { self.start };
                        *surface.lead_sign.entry(earlier).or_default().at(before).at(sign) += 1;
                    }
                    let space = sent[place - 1].end < unit.start;
                    let pair = spaced.at((before, unit.sym));
                    let one = spaced_one.at(unit.sym);
                    if space {
                        pair.0 += 1;
                        one.0 += 1;
                    } else {
                        pair.1 += 1;
                        one.1 += 1;
                    }
                }
            }
            for fact in &lesson.given.facts {
                if self.name_slot[fact.slot as usize] {
                    continue;
                }
                let items: Vec<u32> = if fact.list {
                    fact.items.clone()
                } else if fact.value != WRITE_EMPTY_VALUE && !self.is_list(fact.slot) {
                    vec![fact.value]
                } else {
                    Vec::new()
                };
                for item in items {
                    let places = self.wanted(cache, item).places(text);
                    let forms = value_as.at((fact.slot, item));
                    for (start, end) in places {
                        let shape = self.shapes.number(&text[start..end]);
                        *forms.at(shape) += 1;
                    }
                }
            }
        }
        for (key, forms) in written.rows() {
            if let Some(shape) = forms.first_most().map(|(key, _)| key) {
                surface.written_as.insert(*key, *shape);
            }
        }
        for (key, forms) in value_as.rows() {
            if let Some(shape) = forms.first_most().map(|(key, _)| key) {
                surface.value_as.insert(*key, *shape);
            }
        }
        surface.spaced = spaced.rows().iter().map(|(key, (yes, no))| (*key, yes >= no)).collect();
        surface.spaced_one = spaced_one.rows().iter().map(|(key, (yes, no))| (*key, yes >= no)).collect();
        surface.no_double = (doubled as f64) <= WRITE_DOUBLE_SHARE * (self.rows.len() as f64);
        self.surface = surface;
    }

    pub(crate) fn rank_all(&mut self) {
        let mut order: Vec<u32> = (0..self.syms.len() as u32).collect();
        order.sort_by(|one, other| self.syms.name(*one).cmp(self.syms.name(*other)));
        self.rank = vec![0; order.len()];
        for (place, sym) in order.iter().enumerate() {
            self.rank[*sym as usize] = place as u32;
        }
        let mut order: Vec<u32> = (0..self.vals.len() as u32).collect();
        order.sort_by(|one, other| self.vals.name(*one).cmp(self.vals.name(*other)));
        self.value_rank = vec![0; order.len()];
        for (place, value) in order.iter().enumerate() {
            self.value_rank[*value as usize] = place as u32;
        }
    }
}
