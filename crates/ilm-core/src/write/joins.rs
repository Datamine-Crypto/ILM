use crate::write::forms::{BoundKey, FormStore, SentenceForm, WRITE_ALIKE_SHARE, WRITE_HEAD_AND_NEXT, WRITE_OPEN_UNITS, WRITE_SAME_UNITS, WRITE_USUAL_SENTENCES, canonical_bound};
use crate::write::lessons::{WriterLessons, slot_bit};
use crate::write::reader::TextReader;
use crate::base::first_sight::{FirstSightTable, NumberBag, NumberTable};
use patterns::{because, source};

pub struct JoinedLessonForms;
source!(JoinedLessonForms, "a new form from the start of one lesson sentence and the end of another, when the units before the cut are the same in both, the unit before them is the same or alike, and every run over the cut stands in a lesson form: the lessons then say more forms than they show");

pub const WRITE_JOIN_SENTENCES: i64 = 2;
because!(WRITE_JOIN_SENTENCES, JoinedLessonForms, "the lesson sentences that must start with the head of a joined form, or end with its tail, at least: a start or an end that one lesson alone has can be the slip of one person");

pub const WRITE_JOIN_RUN: usize = 5;
because!(WRITE_JOIN_RUN, JoinedLessonForms, "every run of this many units of a joined form must stand in a lesson form");

pub const WRITE_JOINED_KEPT: usize = 12;
because!(WRITE_JOINED_KEPT, JoinedLessonForms, "the joined forms that are kept for one set of slots and one set of values of their fixed words");

pub const WRITE_JOINED_SLACK: usize = 48;
because!(WRITE_JOINED_SLACK, JoinedLessonForms, "the joined forms of one group that are held while the pairs are tried, before the group is cut back to the kept ones: enough that the cut is rare");

pub const WRITE_PAIRS_MOST: i64 = 1_000_000;
because!(WRITE_PAIRS_MOST, JoinedLessonForms, "the pairs of a head and a tail that are tried at most, a limit of time for many lessons");

#[derive(Clone, Debug, Default)]
struct JoinHead {
    count: i64,
    first: i64,
    later: i64,
    form: u32,
    cut: u32,
}
because!(JoinHead, JoinedLessonForms, "the start of lesson sentences up to a cut: how many sentences start so, as the first sentence of a text and as a later one, and the first form with this start and its cut");

#[derive(Clone, Debug, Default)]
struct JoinTail {
    count: i64,
    form: u32,
    cut: u32,
    before: Vec<u32>,
    was: Vec<BoundKey>,
}
because!(JoinTail, JoinedLessonForms, "the end of lesson sentences after a cut: how many sentences end so, the first form with this end and its cut, the units that stood before the units at the cut, and what the words before the cut said in fixed words in each lesson");

#[derive(Clone, Debug, Default)]
struct JoinPair {
    usual: bool,
    count: i64,
    key: Vec<u32>,
    head: JoinHead,
    tail: JoinTail,
    rarest: i64,
}
because!(JoinPair, JoinedLessonForms, "a head and a tail that passed every test of a join: whether the joined form is usual, the lessons with the head times the lessons with the tail, the units of the joined form, and how many lesson sentences hold its rarest part");

impl FormStore {
    pub(crate) fn learn_joins(&mut self, lessons: &WriterLessons, reader: &TextReader) {
        let mut seen_runs: NumberBag<Vec<u32>> = NumberBag::default();
        for form in &self.forms {
            let padded = FormStore::padded(lessons, &form.key);
            let last = padded.len().saturating_sub(WRITE_JOIN_RUN);
            for at in 0..=last {
                seen_runs.insert(padded[at..(at + WRITE_JOIN_RUN).min(padded.len())].to_vec());
            }
        }
        let mut opened: NumberBag<(Vec<u32>, Vec<u32>, u32)> = NumberBag::default();
        for form in &self.forms {
            let mut key = form.key.clone();
            key.push(lessons.end);
            let open: Vec<u32> = key.iter().take(WRITE_OPEN_UNITS).copied().collect();
            for at in WRITE_SAME_UNITS..key.len() {
                opened.insert((open.clone(), key[at - WRITE_SAME_UNITS..at].to_vec(), key[at]));
            }
        }
        let mut heads: FirstSightTable<(u32, Vec<u32>), JoinHead> = FirstSightTable::new();
        let mut tails: NumberTable<(u32, Vec<u32>), FirstSightTable<Vec<u32>, JoinTail>> = NumberTable::default();
        for (number, form) in self.forms.iter().enumerate() {
            let key = &form.key;
            for cut in WRITE_SAME_UNITS..key.len() {
                let Some((said, _)) = form.side(cut as u32) else {
                    continue;
                };
                let was: BoundKey = canonical_bound(form.fixed.iter().filter(|(slot, _)| said & slot_bit(*slot) != 0).cloned().collect());
                let head = heads.at((form.sign, key[..cut].to_vec()));
                if head.count == 0 {
                    *head = JoinHead { count: form.count, first: form.first, later: form.later, form: number as u32, cut: cut as u32 };
                } else {
                    head.count += form.count;
                    head.first += form.first;
                    head.later += form.later;
                }
                let before = if cut > WRITE_SAME_UNITS { key[cut - WRITE_SAME_UNITS - 1] } else { lessons.start };
                let tail = tails.entry((form.sign, key[cut - WRITE_SAME_UNITS..cut].to_vec())).or_default().at(key[cut..].to_vec());
                if tail.count == 0 {
                    *tail = JoinTail { count: form.count, form: number as u32, cut: cut as u32, before: vec![before], was: vec![was] };
                } else {
                    tail.count += form.count;
                    if !tail.before.contains(&before) {
                        tail.before.push(before);
                    }
                    if !tail.was.contains(&was) {
                        tail.was.push(was);
                    }
                }
            }
        }
        let mut head_counts: NumberTable<(u32, Vec<u32>), Vec<i64>> = NumberTable::default();
        for ((sign, start), head) in heads.rows() {
            head_counts.entry((*sign, start[start.len() - WRITE_SAME_UNITS..].to_vec())).or_default().push(head.count);
        }
        let pairs = |least: i64| -> i64 {
            let mut total = 0i64;
            for (context, counts) in &head_counts {
                let ends: Vec<i64> = tails.get(context).map(|ends| ends.rows().iter().map(|(_, tail)| tail.count).collect()).unwrap_or_default();
                let many = |counts: &[i64], low: i64, high: i64| counts.iter().filter(|count| **count >= low && **count < high).count() as i64;
                total += many(counts, least, i64::MAX) * many(&ends, least, i64::MAX);
                total -= many(counts, least, WRITE_JOIN_SENTENCES) * many(&ends, least, WRITE_JOIN_SENTENCES);
            }
            total
        };
        let mut least = 1i64;
        while pairs(least) > WRITE_PAIRS_MOST {
            least += 1;
        }
        self.join_least = least;
        self.join_pairs = pairs(least);
        let mut kept: FirstSightTable<(u64, BoundKey, u32), Vec<JoinPair>> = FirstSightTable::new();
        for ((sign, start), head) in heads.rows() {
            if head.count < least {
                continue;
            }
            let a = &self.forms[head.form as usize];
            let tied = &self.signs.rows()[*sign as usize].0;
            let plain = reader.carried_order.iter().all(|slot| tied.iter().find(|(seen, _)| seen == slot).map(|(_, value)| *value) == Some(reader.plain[*slot as usize]));
            let said = a.side(head.cut).map_or(0, |(said, _)| said);
            let says: Vec<&(u32, Vec<u32>)> = a.fixed.iter().filter(|(slot, _)| said & slot_bit(*slot) != 0).collect();
            let before = if start.len() > WRITE_SAME_UNITS { start[start.len() - WRITE_SAME_UNITS - 1] } else { lessons.start };
            let context = start[start.len() - WRITE_SAME_UNITS..].to_vec();
            let Some(ends) = tails.get(&(*sign, context.clone())) else {
                continue;
            };
            for (end, tail) in ends.rows() {
                let mut whole = start.clone();
                whole.extend_from_slice(end);
                if tail.count < least || head.count.max(tail.count) < WRITE_JOIN_SENTENCES || self.form_at.contains_key(&(*sign, whole.clone())) {
                    continue;
                }
                if !tail.before.contains(&before) && !tail.before.iter().any(|other| self.alike(before, *other) >= WRITE_ALIKE_SHARE) {
                    continue;
                }
                if !opened.contains(&(start.iter().take(WRITE_OPEN_UNITS).copied().collect(), context.clone(), end[0])) {
                    continue;
                }
                if !plain && !self.start_end.contains(&FormStore::start_and_end(&whole)) {
                    continue;
                }
                let agrees = |was: &BoundKey| was.iter().all(|(slot, values)| says.iter().find(|(seen, _)| seen == slot).is_none_or(|(_, mine)| mine.iter().any(|value| values.contains(value))));
                if !says.is_empty() && !tail.was.iter().any(agrees) {
                    continue;
                }
                let padded = FormStore::padded(lessons, &whole);
                let from = (start.len() + WRITE_HEAD_AND_NEXT).saturating_sub(WRITE_JOIN_RUN);
                let over_cut = padded.len() >= WRITE_JOIN_RUN && (from..=start.len().min(padded.len() - WRITE_JOIN_RUN)).any(|at| !seen_runs.contains(&padded[at..at + WRITE_JOIN_RUN]));
                if over_cut {
                    continue;
                }
                let b = &self.forms[tail.form as usize];
                let more = b.side(tail.cut).map_or(0, |(_, more)| more);
                if said & more != 0 || more == 0 {
                    continue;
                }
                let mut fixed: BoundKey = a.fixed.iter().filter(|(slot, _)| said & slot_bit(*slot) != 0).cloned().collect();
                for (slot, values) in b.fixed.iter().filter(|(slot, _)| more & slot_bit(*slot) != 0) {
                    match fixed.iter_mut().find(|(seen, _)| seen == slot) {
                        Some((_, seen)) => *seen = values.clone(),
                        None => fixed.push((*slot, values.clone())),
                    }
                }
                let group = kept.at((said | more, canonical_bound(fixed), *sign));
                if group.iter().any(|pair| pair.key == whole) {
                    continue;
                }
                let rarest = head.count.min(tail.count).min(self.rarest(lessons, &whole));
                group.push(JoinPair { usual: rarest >= WRITE_USUAL_SENTENCES, count: head.count * tail.count, key: whole, head: head.clone(), tail: tail.clone(), rarest });
                if group.len() > WRITE_JOINED_SLACK {
                    FormStore::best_pairs(lessons, group);
                }
            }
        }
        let mut joined: Vec<SentenceForm> = Vec::new();
        for ((slots, _, sign), group) in kept.rows_mut() {
            FormStore::best_pairs(lessons, group);
            for pair in group.iter() {
                let a = &self.forms[pair.head.form as usize];
                let b = &self.forms[pair.tail.form as usize];
                let (i, j) = (pair.head.cut, pair.tail.cut);
                let said = a.side(i).map_or(0, |(said, _)| said);
                let more = b.side(j).map_or(0, |(_, more)| more);
                let tied = self.signs.rows()[*sign as usize].0.clone();
                let mut form = SentenceForm { key: pair.key.clone(), count: pair.count, first: pair.head.first, later: pair.head.later, usual: pair.rarest, sign: *sign, ..SentenceForm::default() };
                for slot in 0..lessons.slots.len() as u32 {
                    let everywhere = pair.tail.was.iter().all(|was| was.iter().any(|(seen, _)| *seen == slot));
                    if !everywhere || (said | more) & slot_bit(slot) != 0 {
                        continue;
                    }
                    let mut values: Vec<u32> = pair.tail.was.iter().flat_map(|was| was.iter().filter(|(seen, _)| *seen == slot).flat_map(|(_, found)| found.iter().copied())).collect();
                    values.sort();
                    values.dedup();
                    let with = &self.values_with[slot as usize];
                    let only = b.key.iter().enumerate().any(|(at, unit)| {
                        at as u32 >= j && !lessons.label_slot.contains_key(unit) && !lessons.is_list(slot) && with.get(unit).is_some_and(|seen| !seen.rows().is_empty() && seen.rows().iter().all(|(value, _)| values.contains(value)))
                    });
                    if only {
                        form.needs.push((slot, values));
                    }
                }
                form.rare = self.rare_runs(lessons, &pair.key) + i64::from(pair.head.count < WRITE_USUAL_SENTENCES) + i64::from(pair.tail.count < WRITE_USUAL_SENTENCES);
                form.places = a.places.iter().filter(|(at, _)| *at < i).copied().collect();
                form.places.extend(b.places.iter().filter(|(at, _)| *at >= j).map(|(at, slot)| (*at + i - j, *slot)));
                form.which = a.which[..i as usize].to_vec();
                form.which.extend_from_slice(&b.which[j as usize..]);
                form.slots = *slots | tied.iter().fold(0u64, |mask, (slot, _)| mask | slot_bit(*slot));
                form.tied = tied;
                form.fixed = a.fixed.iter().filter(|(slot, _)| said & slot_bit(*slot) != 0).cloned().collect();
                for (slot, values) in b.fixed.iter().filter(|(slot, _)| more & slot_bit(*slot) != 0) {
                    match form.fixed.iter_mut().find(|(seen, _)| seen == slot) {
                        Some((_, seen)) => *seen = values.clone(),
                        None => form.fixed.push((*slot, values.clone())),
                    }
                }
                form.lesson = (a.lesson.0, b.lesson.0);
                form.parts = Some((pair.head.form, i, pair.tail.form, j));
                for (slot, values) in &a.after {
                    let both = match b.after.iter().find(|(seen, _)| seen == slot) {
                        Some((_, other)) => values.iter().copied().filter(|value| other.contains(value)).collect(),
                        None => values.clone(),
                    };
                    form.after.push((*slot, both));
                }
                for (slot, values) in &b.after {
                    if !a.after.iter().any(|(seen, _)| seen == slot) {
                        form.after.push((*slot, values.clone()));
                    }
                }
                joined.push(form);
            }
        }
        self.forms.extend(joined);
    }

    fn best_pairs(lessons: &WriterLessons, group: &mut Vec<JoinPair>) {
        group.sort_by(|one, other| other.usual.cmp(&one.usual).then(other.count.cmp(&one.count)).then_with(|| lessons.compare_units(&one.key, &other.key)));
        group.truncate(WRITE_JOINED_KEPT);
    }
}
