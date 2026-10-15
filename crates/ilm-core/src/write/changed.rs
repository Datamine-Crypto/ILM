use crate::base::keys::NO_NUMBER;
use crate::write::desk::WritingDesk;
use crate::write::forms::{FormStore, SentenceForm, WRITE_HEAD_AND_NEXT, WRITE_USUAL_SENTENCES};
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, WRITE_LONGEST_RUN, slot_bit};
use crate::write::phrases::{WRITE_BESIDE_UNITS, WRITE_EDIT_RUN};
use crate::base::first_sight::FirstSightCount;
use patterns::source;

pub struct ChangedForms;
source!(ChangedForms, "a lesson form changed by one cut, for a list of facts that no sequence of forms says: a run of units that says a slot the list does not have goes out, and the phrase of a missing slot goes in where the lessons have that phrase between the same units; the reader must read the changed form back to the given facts, and of the cuts that read back the smallest is kept");

impl<'a> WritingDesk<'a> {
    fn stands_without(&mut self, run: u32, slot: u32) -> i64 {
        if let Some(known) = self.without.get(&(run, slot)) {
            return *known;
        }
        let reader = self.filler.reader;
        let many = reader.holds[run as usize].iter().filter(|sentence| !reader.read(&reader.sent_toks[**sentence as usize]).iter().any(|found| reader.keys[found.key as usize].slot == slot)).count() as i64;
        self.without.insert((run, slot), many);
        many
    }

    fn left_over(&mut self, units: &[u32], slot: u32, given: &GivenFacts) -> bool {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        for size in 1..=WRITE_LONGEST_RUN {
            for part in units.windows(size) {
                let Some(run) = reader.run_number(part) else {
                    continue;
                };
                let key = reader.owner[run as usize];
                if key == NO_NUMBER || reader.keys[key as usize].slot != slot {
                    continue;
                }
                let tied = match self.tied_to.get(&(run, key)) {
                    Some(known) => *known,
                    None => {
                        let mut texts: Vec<u32> = reader.holds[run as usize].iter().map(|sentence| reader.sent_lesson[*sentence as usize]).collect();
                        texts.dedup();
                        let mut seen: FirstSightCount<u32> = FirstSightCount::new();
                        for lesson in texts {
                            *seen.at(reader.value_of[key as usize][lesson as usize]) += 1;
                        }
                        let text = |value: u32| if value == NO_NUMBER { "None" } else { lessons.vals.name(value) };
                        let mut best: Option<(u32, i64)> = None;
                        for (value, many) in seen.rows() {
                            if best.is_none_or(|(top, count)| *many > count || (*many == count && text(*value) < text(top))) {
                                best = Some((*value, *many));
                            }
                        }
                        let tied = best.map_or(NO_NUMBER, |(value, _)| value);
                        self.tied_to.insert((run, key), tied);
                        tied
                    }
                };
                if tied == NO_NUMBER {
                    continue;
                }
                let same = given.fact(slot).is_some_and(|fact| if fact.list { fact.items.contains(&tied) } else { fact.value == tied });
                if !same {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn changed_forms(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)], lost: u64, loose: u32) -> Vec<u32> {
        let memo = (lost, loose, self.filler.name_forms);
        if let Some(known) = self.changed_kept.get(&memo) {
            return known.clone();
        }
        let made = self.changed_new(given, names, lost, loose);
        self.changed_kept.insert(memo, made.clone());
        made
    }

    fn changed_new(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)], lost: u64, loose: u32) -> Vec<u32> {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        let forms = self.filler.forms;
        let numbers: Vec<(u32, u32)> = given.facts.iter().filter(|fact| fact.list).map(|fact| (fact.slot, fact.items.len() as u32)).collect();
        let mut out: Vec<u32> = Vec::new();
        let mut made: Vec<(u32, Vec<u32>)> = Vec::new();
        for (number, form) in forms.forms[..forms.lesson_forms].iter().enumerate() {
            let bound = form.tied.iter().all(|(slot, value)| given.value(*slot) == Some(*value) || (loose > 0 && given.has(*slot) && *value == self.filler.plain_of(*slot).unwrap_or(WRITE_EMPTY_VALUE)));
            if !bound {
                continue;
            }
            let key = &form.key;
            let spans = form.phrase_spans();
            let span = |slot: u32| spans.iter().find(|(seen, _)| *seen == slot).map_or((0, 0), |(_, span)| *span);
            let wrong: Vec<u32> = spans.iter().map(|(slot, _)| *slot).filter(|slot| !given.has(*slot) || form.fixed_of(*slot).is_some_and(|values| !given.value(*slot).is_some_and(|value| values.contains(&value)))).collect();
            let keeps: Vec<u32> = spans.iter().map(|(slot, _)| *slot).filter(|slot| !wrong.contains(slot)).collect();
            let padded = FormStore::padded(lessons, key);
            let cuts: Vec<(usize, usize)> = if wrong.is_empty() {
                (0..key.len()).map(|at| (at, at)).collect()
            } else {
                let mut taken = vec![false; key.len() + 1];
                for slot in &keeps {
                    let (from, to) = span(*slot);
                    for at in from..to {
                        taken[at as usize] = true;
                    }
                }
                let mut cover_all = vec![false; key.len() + 1];
                for (_, (from, to)) in &spans {
                    for at in *from..*to {
                        cover_all[at as usize] = true;
                    }
                }
                let mut cover = vec![false; key.len() + 1];
                let mut low = usize::MAX;
                let mut high = 0usize;
                for slot in &wrong {
                    let (a, b) = span(*slot);
                    let (a, b) = (a as usize, b as usize);
                    let mut left_end = a;
                    for left in 1..=WRITE_BESIDE_UNITS {
                        if a < left || cover_all[a - left] || self.filler.says_carried(form, key[a - left]) {
                            break;
                        }
                        left_end = a - left;
                    }
                    let mut right_end = b;
                    for right in 1..=WRITE_BESIDE_UNITS {
                        if b + right > key.len() - 1 || cover_all[b + right - 1] || self.filler.says_carried(form, key[b + right - 1]) {
                            break;
                        }
                        right_end = b + right;
                    }
                    for at in left_end..right_end {
                        cover[at] = true;
                    }
                    low = low.min(left_end);
                    high = high.max(right_end);
                }
                let high = high.min(key.len() - 1);
                let mut cuts: Vec<(usize, usize)> = Vec::new();
                for a in low..high {
                    for b in a + 1..=high {
                        let free = (a..b).all(|at| cover[at] && !taken[at]);
                        let touches = wrong.iter().all(|slot| {
                            let (from, to) = span(*slot);
                            a < to as usize && (from as usize) < b
                        });
                        if free && touches {
                            cuts.push((a, b));
                        }
                    }
                }
                cuts
            };
            let first_wrong = wrong.iter().map(|slot| span(*slot).0 as usize).min().unwrap_or(0);
            let last_wrong = wrong.iter().map(|slot| span(*slot).1 as usize).max().unwrap_or(0);
            let part: Vec<u32> = reader.carried_order.iter().map(|slot| form.tied_of(*slot).unwrap_or(NO_NUMBER)).collect();
            let pairs = forms.pairs_after.get(&(part, key[0]));
            let mut found: Vec<(u32, Vec<(usize, Vec<u32>, Vec<u32>)>)> = Vec::new();
            for (a, b) in cuts {
                let whole: Vec<u32> = keeps
                    .iter()
                    .copied()
                    .filter(|slot| {
                        let (from, to) = span(*slot);
                        !(a < to as usize && (from as usize) < b)
                    })
                    .collect();
                let whole_mask = whole.iter().fold(0u64, |mask, slot| mask | slot_bit(*slot));
                let mut depends = false;
                for slot in &whole {
                    if form.fixed_of(*slot).is_none() {
                        continue;
                    }
                    let places = form.said_at.iter().find(|(seen, _)| seen == slot).map_or(&[][..], |(_, places)| places.as_slice());
                    for (from, to) in places {
                        let Some(run) = reader.run_number(&key[*from as usize..*to as usize]).filter(|run| !reader.holds[*run as usize].is_empty()) else {
                            continue;
                        };
                        for gone in &wrong {
                            if self.stands_without(run, *gone) < WRITE_USUAL_SENTENCES {
                                depends = true;
                            }
                        }
                    }
                }
                if depends {
                    continue;
                }
                let mut options: Vec<(u32, Vec<u32>, Vec<u32>, bool)> = Vec::new();
                if b > a && whole_mask & lost != 0 && a >= first_wrong && b <= last_wrong {
                    options.push((NO_NUMBER, Vec::new(), Vec::new(), false));
                }
                if let Some(phrases) = forms.between.get(&(padded[a], padded[b + 1])) {
                    for ((slot, units), phrase) in phrases.rows() {
                        let fits = lost & slot_bit(*slot) != 0 && !whole.contains(slot) && given.has(*slot) && (phrase.values.is_empty() || given.value(*slot).is_some_and(|value| phrase.values.contains(&value)));
                        if !fits {
                            continue;
                        }
                        let holds = |run: &[u32]| pairs.is_some_and(|pairs| pairs.contains(run));
                        if !wrong.is_empty() && phrase.beside.0 > 0 && a > 0 && a < first_wrong && !holds(&[key[a - 1], units[0]]) {
                            continue;
                        }
                        if !wrong.is_empty() && phrase.beside.1 > 0 && b < key.len() && b > last_wrong {
                            let mut run: Vec<u32> = units[units.len().saturating_sub(WRITE_HEAD_AND_NEXT)..].to_vec();
                            run.push(key[b]);
                            if !holds(&run) {
                                continue;
                            }
                        }
                        options.push((*slot, units.clone(), phrase.places.clone(), !phrase.values.is_empty()));
                    }
                }
                for (slot, units, places, bound_value) in options {
                    let mut new_key: Vec<u32> = key[..a].to_vec();
                    new_key.extend_from_slice(&units);
                    new_key.extend_from_slice(&key[b..]);
                    if forms.form_at.contains_key(&(form.sign, new_key.clone())) || made.iter().any(|(sign, seen)| *sign == form.sign && *seen == new_key) {
                        continue;
                    }
                    let mut stays = false;
                    for gone in &wrong {
                        if self.left_over(&key[..a], *gone, given) || self.left_over(&key[b..], *gone, given) {
                            stays = true;
                        }
                    }
                    if stays {
                        continue;
                    }
                    if units.is_empty() && !forms.start_end.contains(&FormStore::start_and_end(&new_key)) {
                        continue;
                    }
                    let new = FormStore::padded(lessons, &new_key);
                    let seen = |from: usize, to: usize| forms.runs_seen.contains(&new[from..to.min(new.len())]);
                    if units.is_empty() || a == b {
                        let from = (a + WRITE_HEAD_AND_NEXT).saturating_sub(WRITE_EDIT_RUN);
                        let last = (a + units.len()).min(new.len().wrapping_sub(WRITE_EDIT_RUN));
                        if new.len() >= WRITE_EDIT_RUN && (from..=last).any(|at| !seen(at, at + WRITE_EDIT_RUN)) {
                            continue;
                        }
                    } else {
                        if a + WRITE_HEAD_AND_NEXT >= WRITE_EDIT_RUN && !seen(a + WRITE_HEAD_AND_NEXT - WRITE_EDIT_RUN, a + WRITE_HEAD_AND_NEXT) {
                            continue;
                        }
                        if a + units.len() + WRITE_EDIT_RUN <= new.len() && !seen(a + units.len(), a + units.len() + WRITE_EDIT_RUN) {
                            continue;
                        }
                    }
                    let shift = |at: u32| (at as usize + units.len() - (b - a)) as u32;
                    let mut new_places: Vec<(u32, u32)> = form.places.iter().copied().filter(|(at, _)| (*at as usize) < a).collect();
                    new_places.extend(places.iter().map(|at| (a as u32 + *at, slot)));
                    new_places.extend(form.places.iter().filter(|(at, _)| *at as usize >= b).map(|(at, seen)| (shift(*at), *seen)));
                    new_places.sort_by(|one, other| one.0.cmp(&other.0).then_with(|| lessons.slots.name(one.1).cmp(lessons.slots.name(other.1))));
                    let mut which: Vec<u32> = form.which[..a].to_vec();
                    which.extend(std::iter::repeat_n(NO_NUMBER, units.len()));
                    which.extend_from_slice(&form.which[b..]);
                    if slot != NO_NUMBER && lessons.is_list(slot) {
                        for (item, at) in places.iter().enumerate() {
                            which[a + *at as usize] = item as u32;
                        }
                    }
                    let mut said_at: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
                    for (at, seen) in &new_places {
                        match said_at.iter_mut().find(|(known, _)| known == seen) {
                            Some((_, spans)) => spans.push((*at, *at + 1)),
                            None => said_at.push((*seen, vec![(*at, *at + 1)])),
                        }
                    }
                    let mut fixed: Vec<(u32, Vec<u32>)> = form.fixed.iter().filter(|(seen, _)| whole.contains(seen)).cloned().collect();
                    if bound_value {
                        let value = given.value(slot).unwrap_or(WRITE_EMPTY_VALUE);
                        match fixed.iter_mut().find(|(seen, _)| *seen == slot) {
                            Some((_, values)) => *values = vec![value],
                            None => fixed.push((slot, vec![value])),
                        }
                    }
                    let mut slots = whole_mask | form.tied_mask();
                    if slot != NO_NUMBER {
                        slots |= slot_bit(slot);
                    }
                    let changed = SentenceForm {
                        usual: form.usual.min(forms.rarest(lessons, &new_key)),
                        rare: forms.rare_runs(lessons, &new_key),
                        key: new_key.clone(),
                        places: new_places,
                        which,
                        said_at,
                        fixed,
                        tied: form.tied.clone(),
                        sign: form.sign,
                        slots,
                        count: form.count,
                        first: form.first,
                        later: form.later,
                        lesson: form.lesson,
                        after: form.after.clone(),
                        edited: Some((number as u32, key[a..b].to_vec(), units.clone())),
                        needs: Vec::new(),
                        parts: None,
                        resized: None,
                    };
                    let mut has: Vec<(u32, u32)> = Vec::new();
                    for (at, seen) in &changed.places {
                        if changed.item_at(*at as usize).is_some() {
                            match has.iter_mut().find(|(known, _)| known == seen) {
                                Some((_, many)) => *many += 1,
                                None => has.push((*seen, 1)),
                            }
                        }
                    }
                    let change: Vec<(u32, u32)> = has.iter().filter_map(|(seen, many)| numbers.iter().find(|(known, wanted)| known == seen && wanted != many).copied()).collect();
                    let kept = self.keep(changed);
                    let finals: Vec<u32> = if change.is_empty() { vec![kept] } else { self.resized(kept, &change) };
                    let mut good: Vec<u32> = Vec::new();
                    for last in finals {
                        let Some(filled) = self.fill(last, given, loose) else {
                            continue;
                        };
                        let Some(said) = self.said_by(last, given, names, &filled) else {
                            continue;
                        };
                        let made_form = self.form(last);
                        if made_form.slots & !(said | made_form.tied_mask()) == 0 {
                            good.push(last);
                        }
                    }
                    if !good.is_empty() {
                        match found.iter_mut().find(|(seen, _)| *seen == slot) {
                            Some((_, rows)) => rows.push((b - a, new_key, good)),
                            None => found.push((slot, vec![(b - a, new_key, good)])),
                        }
                    }
                }
            }
            for (_, rows) in found {
                let least = rows.iter().map(|(size, _, _)| *size).min().unwrap_or(0);
                for (size, new_key, good) in rows {
                    if size == least && !made.iter().any(|(sign, seen)| *sign == form.sign && *seen == new_key) {
                        made.push((form.sign, new_key));
                        out.extend(good);
                    }
                }
            }
        }
        out
    }
}
