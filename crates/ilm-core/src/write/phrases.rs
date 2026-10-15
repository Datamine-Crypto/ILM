use crate::base::keys::NO_NUMBER;
use crate::write::forms::{FormStore, ListSides, ListWay, SlotPhrase, WRITE_HEAD_AND_NEXT, WRITE_USUAL_SENTENCES};
use crate::write::lessons::WriterLessons;
use crate::write::reader::TextReader;
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag, key_count};
use patterns::{because, source};

pub struct ListWords;
source!(ListWords, "the words between the items of a list and before its last item are counted from the lessons, and so is the phrase that says one slot with the units beside it; a list of another number of items and a changed form are made from these");

pub const WRITE_LIST_MANY: usize = 3;
because!(WRITE_LIST_MANY, ListWords, "the items from which a list has words between its items and other words before its last item");

pub const WRITE_EDIT_RUN: usize = 3;
because!(WRITE_EDIT_RUN, ListWords, "a lesson form is changed only when the runs of this many units over the cut stand in a lesson form");

pub const WRITE_BESIDE_UNITS: usize = 2;
because!(WRITE_BESIDE_UNITS, ListWords, "the units beside the words of a slot that go into its phrase, or out of a form with them, at most");

impl FormStore {
    pub(crate) fn learn_list_words(&mut self, lessons: &WriterLessons) {
        let mut shapes: Vec<(usize, u32, Vec<usize>, Vec<Vec<u32>>)> = Vec::new();
        for (number, form) in self.forms[..self.lesson_forms].iter().enumerate() {
            let mut slots: Vec<u32> = form.places.iter().filter(|(at, _)| form.item_at(*at as usize).is_some()).map(|(_, slot)| *slot).collect();
            slots.sort_by(|one, other| lessons.slots.name(*one).cmp(lessons.slots.name(*other)));
            slots.dedup();
            for slot in slots {
                let Some((ats, gaps)) = form.list_shape(slot) else {
                    continue;
                };
                if ats.len() >= WRITE_LIST_MANY {
                    let mut once: Vec<&Vec<u32>> = Vec::new();
                    for gap in &gaps {
                        if !once.contains(&gap) {
                            once.push(gap);
                            *self.list_gaps.entry(gap.clone()).or_insert(0) += form.count;
                        }
                    }
                }
                shapes.push((number, slot, ats, gaps));
            }
        }
        for (number, slot, ats, gaps) in &shapes {
            let form = &self.forms[*number];
            if ats.len() < WRITE_HEAD_AND_NEXT || gaps.iter().any(|gap| self.list_gaps.get(gap).copied().unwrap_or(0) < WRITE_USUAL_SENTENCES) {
                continue;
            }
            let left = if ats[0] > 0 { form.key[ats[0] - 1] } else { lessons.start };
            let last = ats[ats.len() - 1];
            let right = if last + 1 < form.key.len() { form.key[last + 1] } else { lessons.end };
            let seen = if ats.len() == WRITE_HEAD_AND_NEXT { self.list_two.at(*slot).at((Vec::new(), gaps[0].clone())) } else { self.list_more.at(*slot).at((gaps[0].clone(), gaps[gaps.len() - 1].clone())) };
            for sides in [(left, right), (NO_NUMBER, right), (left, NO_NUMBER), (NO_NUMBER, NO_NUMBER)] {
                *seen.at(sides) += form.count;
            }
        }
        for (_, found) in self.list_two.rows() {
            for ((_, gap), seen) in found.rows() {
                *self.list_pair.entry(gap.clone()).or_insert(0) += key_count(seen, &(NO_NUMBER, NO_NUMBER));
            }
        }
        for (_, found) in self.list_more.rows() {
            for ((inner, gap), seen) in found.rows() {
                if gap != inner {
                    *self.list_last.at(gap.clone()) += key_count(seen, &(NO_NUMBER, NO_NUMBER));
                }
            }
        }
        for table in [&mut self.list_two, &mut self.list_more] {
            let mut shared: FirstSightCount<ListWay> = FirstSightCount::new();
            for (_, found) in table.rows() {
                for (way, _) in found.rows() {
                    *shared.at(way.clone()) += 1;
                }
            }
            let mut common: FirstSightTable<ListWay, FirstSightCount<ListSides>> = FirstSightTable::new();
            for (_, found) in table.rows() {
                for (way, seen) in found.rows() {
                    if key_count(&shared, way) > 1 {
                        let all = common.at(way.clone());
                        for (sides, count) in seen.rows() {
                            *all.at(*sides) += *count;
                        }
                    }
                }
            }
            table.put(NO_NUMBER, common);
        }
    }

    pub(crate) fn learn_phrases(&mut self, lessons: &WriterLessons, reader: &TextReader) {
        let mut between: FirstSightTable<ListSides, FirstSightTable<(u32, Vec<u32>), SlotPhrase>> = FirstSightTable::new();
        let mut count: FirstSightCount<(u32, Vec<u32>)> = FirstSightCount::new();
        let mut bare: NumberBag<(u32, Vec<u32>)> = NumberBag::default();
        for form in &self.forms[..self.lesson_forms] {
            let padded = FormStore::padded(lessons, &form.key);
            for at in 0..(padded.len() + 1).saturating_sub(WRITE_EDIT_RUN) {
                self.runs_seen.insert(padded[at..at + WRITE_EDIT_RUN].to_vec());
            }
            let spans = form.phrase_spans();
            let mut taken = vec![false; form.key.len() + 1];
            for (_, (from, to)) in &spans {
                for at in *from..*to {
                    taken[at as usize] = true;
                }
            }
            for (slot, (a, b)) in &spans {
                if spans.iter().any(|(other, (c, d))| other != slot && a < d && c < b) {
                    continue;
                }
                if lessons.is_list(*slot) && form.list_shape(*slot).is_none() {
                    continue;
                }
                let (a, b) = (*a as usize, *b as usize);
                for left in 0..=WRITE_BESIDE_UNITS {
                    if left > 0 && (a < left || taken[a - left]) {
                        break;
                    }
                    for right in 0..=WRITE_BESIDE_UNITS {
                        if right > 0 && (b + right > form.key.len() - 1 || taken[b + right - 1]) {
                            break;
                        }
                        let (i, j) = (a - left, b + right);
                        let phrase = (*slot, form.key[i..j].to_vec());
                        let found = between.at((padded[i], padded[j + 1]));
                        if !found.holds(&phrase) {
                            let places = form.places.iter().filter(|(_, seen)| seen == slot).map(|(at, _)| *at - i as u32).collect();
                            found.put(phrase.clone(), SlotPhrase { count: 0, places, values: Vec::new(), beside: (left as u32, right as u32) });
                        }
                        let entry = found.at(phrase.clone());
                        entry.count += form.count;
                        for value in form.fixed_of(*slot).map_or(&[][..], |values| values.as_slice()) {
                            if !entry.values.contains(value) {
                                entry.values.push(*value);
                            }
                        }
                        *count.at(phrase.clone()) += form.count;
                        if left == 0 && right == 0 {
                            bare.insert(phrase);
                        }
                    }
                }
            }
        }
        for form in &self.forms[..self.lesson_forms] {
            let part: Vec<u32> = reader.carried_order.iter().map(|slot| form.tied_of(*slot).unwrap_or(NO_NUMBER)).collect();
            let seen = self.pairs_after.entry((part, form.key[0])).or_default();
            for size in [WRITE_HEAD_AND_NEXT, WRITE_EDIT_RUN] {
                for run in form.key.windows(size) {
                    seen.insert(run.to_vec());
                }
            }
        }
        for (sides, found) in between.rows() {
            let mut kept: FirstSightTable<(u32, Vec<u32>), SlotPhrase> = FirstSightTable::new();
            for (phrase, entry) in found.rows() {
                if bare.contains(phrase) || key_count(&count, phrase) >= WRITE_USUAL_SENTENCES {
                    kept.put(phrase.clone(), entry.clone());
                }
            }
            self.between.put(*sides, kept);
        }
    }
}
