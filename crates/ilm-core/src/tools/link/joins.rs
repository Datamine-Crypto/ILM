use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberTable};
use crate::base::symbols::Symbols;
use patterns::{because, source};
use std::cmp::{Ordering, Reverse};

pub struct LinkJoins;
source!(LinkJoins, "a join of two links, counted from lessons that give only the story, the question and the answer: a table from a pair of labels to an answer. Step zero is counted on the lessons whose way has two links; in a longer lesson, two parts side by side with no entry for their pair are tried as one part with each answer of the lessons as its label, and a pair is an entry when one answer makes enough lessons come out right and no other answer makes any; this is done again with the new entries, each with the number of its round as its step. The labels of a way are joined in every order of cuts, as a chart of a sum of brackets is filled, and the label of the whole way is the one that needs the lowest step, then the one with more orders of cuts, then the more frequent answer of the lessons");

pub(super) const LINK_LEAST: u32 = 2;
because!(LINK_LEAST, LinkJoins, "an entry of the join table counts from this many readings, and a new entry of a later step from this many lessons; one reading can be chance");

pub(super) const LINK_SURE_SHARE: f64 = 0.98;
because!(LINK_SURE_SHARE, LinkJoins, "the least share of the readings of an entry that must give its answer: a fault of a set may break a rare lesson, and not more");

pub(super) const LINK_PAIR: usize = 2;
because!(LINK_PAIR, LinkJoins, "the links of the shortest way that is joined: step zero counts the ways of this many links, and only a longer way has parts of more than one link");

pub(super) const LINK_LATER_STEPS: u32 = 12;
because!(LINK_LATER_STEPS, LinkJoins, "the most steps of the join table after step zero: a way of the lessons is seldom longer, and each step reads every longer lesson again");

pub(crate) const LINK_LENT_STEP: u32 = LINK_LATER_STEPS + 1;
because!(LINK_LENT_STEP, AlikeJoins, "the step of a lent join, after every counted step, so that a lent join is used only where no counted join reaches the whole way");

#[derive(Clone, Debug, Default)]
pub(super) struct Joins {
    pub(super) table: FirstSightTable<(u32, u32), u32>,
    pub(super) step: NumberTable<(u32, u32), u32>,
    pub(super) single: FirstSightTable<u32, u32>,
}
because!(Joins, LinkJoins, "the join table with the step of each entry, and for a way of one link the answer that its label gives");

pub(super) type LinkSpan = Vec<(u32, u32, u64)>;
because!(LinkSpan, LinkJoins, "the labels of one part of a way: each with the lowest step that joins it and the orders of cuts that give it at that step, in the order of first sight");

#[derive(Clone, Debug, Default)]
pub(super) struct LinkLessons {
    pub(super) lessons: Vec<(Vec<u32>, u32)>,
    pub(super) results: Vec<u32>,
    pub(super) often: NumberTable<u32, u32>,
}
because!(LinkLessons, LinkJoins, "the lessons with a way: the labels of the links of each way with the answer of the lesson; every answer of the lessons in the order of its text, and how often each answer stands. chart gives the labels of every part of a way; best gives the label of a part; count gives the join table for labels that are one label so far, and the lessons of more than one link that come out right; rows gives the joins of each label");

pub(super) fn chart_by(labels: &[u32], mut find: impl FnMut(u32, u32) -> Option<(u32, u32)>) -> Vec<LinkSpan> {
    let size = labels.len() + 1;
    let mut span: Vec<LinkSpan> = vec![Vec::new(); size * size];
    for (at, label) in labels.iter().enumerate() {
        span[at * size + at + 1] = vec![(*label, 0, 1)];
    }
    for width in LINK_PAIR..size {
        for first in 0..size - width {
            let mut found: LinkSpan = Vec::new();
            for cut in first + 1..first + width {
                for (left, left_step, left_ways) in &span[first * size + cut] {
                    for (right, right_step, right_ways) in &span[cut * size + first + width] {
                        let Some((got, entry_step)) = find(*left, *right) else { continue };
                        let step = entry_step.max(*left_step).max(*right_step);
                        let ways = left_ways.saturating_mul(*right_ways);
                        match found.iter_mut().find(|entry| entry.0 == got) {
                            None => found.push((got, step, ways)),
                            Some(old) if step < old.1 => *old = (got, step, ways),
                            Some(old) if step == old.1 => old.2 = old.2.saturating_add(ways),
                            Some(_) => {}
                        }
                    }
                }
            }
            span[first * size + first + width] = found;
        }
    }
    span
}
because!(chart_by, LinkJoins, "the labels of every part of a way, each with its lowest step and its orders of cuts, where find gives the answer and the step of the join of two labels: the chart of a sum of brackets, filled from the shortest parts to the whole way");

pub(super) fn joined_entry(joins: &Joins, left: u32, right: u32) -> Option<(u32, u32)> {
    joins.table.get(&(left, right)).map(|got| (*got, joins.step.get(&(left, right)).copied().unwrap_or(0)))
}
because!(joined_entry, LinkJoins, "the answer of the join of two labels in a join table with the step of the entry, step zero for an entry with no later step");

impl LinkLessons {
    pub(super) fn chart(&self, labels: &[u32], joins: &Joins) -> Vec<LinkSpan> {
        chart_by(labels, |left, right| joined_entry(joins, left, right))
    }

    pub(super) fn chart_counted(&self, labels: &[u32], joins: &Joins) -> Vec<LinkSpan> {
        chart_by(labels, |left, right| joined_entry(joins, left, right).filter(|(_, step)| *step < LINK_LENT_STEP))
    }

    pub(super) fn best(&self, found: &LinkSpan, items: &Symbols) -> Option<u32> {
        let key = |entry: &(u32, u32, u64)| (entry.1, Reverse(entry.2), Reverse(self.often.get(&entry.0).copied().unwrap_or(0)));
        let mut order: Vec<&(u32, u32, u64)> = found.iter().collect();
        order.sort_by(|left, right| key(left).cmp(&key(right)).then_with(|| items.name(left.0).cmp(items.name(right.0))));
        match order.as_slice() {
            [] => None,
            [first, second, ..] if key(first) == key(second) => None,
            [first, ..] => Some(first.0),
        }
    }

    pub(super) fn count(&self, same: &dyn Fn(u32) -> u32, later: bool, items: &Symbols) -> (Joins, Vec<bool>) {
        let mut base: FirstSightTable<(u32, u32), FirstSightCount<u32>> = FirstSightTable::new();
        let mut single: FirstSightTable<u32, FirstSightCount<u32>> = FirstSightTable::new();
        let mut longer: Vec<(Vec<u32>, u32)> = Vec::new();
        let mut both: Vec<(usize, Vec<u32>, u32)> = Vec::new();
        for (number, (labels, answer)) in self.lessons.iter().enumerate() {
            let labels: Vec<u32> = labels.iter().map(|label| same(*label)).collect();
            match labels.as_slice() {
                [one] => single.at(*one).add(*answer, 1),
                [one, other] => {
                    base.at((*one, *other)).add(*answer, 1);
                    both.push((number, labels, *answer));
                }
                _ => {
                    longer.push((labels.clone(), *answer));
                    both.push((number, labels, *answer));
                }
            }
        }
        let mut joins = Joins::default();
        for (key, counts) in base.rows() {
            if let Some(answer) = link_sure(counts) {
                joins.table.put(*key, answer);
            }
        }
        for (key, counts) in single.rows() {
            if let Some(answer) = link_sure(counts) {
                joins.single.put(*key, answer);
            }
        }
        for round in 1..=if later { LINK_LATER_STEPS } else { 0 } {
            let mut readings: FirstSightTable<(u32, u32), FirstSightTable<u32, Vec<usize>>> = FirstSightTable::new();
            for (number, (labels, answer)) in longer.iter().enumerate() {
                let size = labels.len() + 1;
                let span = self.chart(labels, &joins);
                for cut in 1..labels.len() {
                    for (left, _, _) in &span[cut] {
                        for (right, _, _) in &span[cut * size + labels.len()] {
                            if joins.table.holds(&(*left, *right)) {
                                continue;
                            }
                            let held = readings.at((*left, *right)).at(*answer);
                            if held.last() != Some(&number) {
                                held.push(number);
                            }
                        }
                    }
                }
            }
            let new: Vec<((u32, u32), u32)> = readings.rows().iter().filter_map(|(key, by_result)| match by_result.rows() {
                [(result, lessons)] if !lessons.is_empty() => Some((*key, *result)),
                _ => None,
            }).collect();
            if new.is_empty() {
                break;
            }
            for (key, result) in new {
                joins.table.put(key, result);
                joins.step.insert(key, round);
            }
        }
        if later {
            for (key, answer) in alike_joins(&joins) {
                joins.table.put(key, answer);
                joins.step.insert(key, LINK_LENT_STEP);
            }
        }
        let mut right = vec![false; self.lessons.len()];
        for (number, labels, answer) in &both {
            let span = self.chart(labels, &joins);
            right[*number] = self.best(&span[labels.len()], items) == Some(*answer);
        }
        (joins, right)
    }

    pub(super) fn rows(&self, same: &dyn Fn(u32) -> u32, joins: &Joins, items: &Symbols) -> FirstSightTable<u32, FirstSightTable<(bool, u32), u32>> {
        let mut cuts: FirstSightTable<(u32, u32), FirstSightCount<u32>> = FirstSightTable::new();
        for (labels, answer) in &self.lessons {
            if labels.len() <= LINK_PAIR {
                continue;
            }
            let labels: Vec<u32> = labels.iter().map(|label| same(*label)).collect();
            let size = labels.len() + 1;
            let span = self.chart(&labels, joins);
            for cut in 1..labels.len() {
                if let (Some(left), Some(right)) = (self.best(&span[cut], items), self.best(&span[cut * size + labels.len()], items)) {
                    cuts.at((left, right)).add(*answer, 1);
                }
            }
        }
        let mut rows: FirstSightTable<u32, FirstSightTable<(bool, u32), u32>> = FirstSightTable::new();
        for ((left, right), counts) in cuts.rows() {
            if let Some(answer) = link_sure(counts) {
                rows.at(*left).put((true, *right), answer);
                rows.at(*right).put((false, *left), answer);
            }
        }
        for ((left, right), answer) in joins.table.rows() {
            rows.at(*left).put((true, *right), *answer);
            rows.at(*right).put((false, *left), *answer);
        }
        rows
    }
}

pub struct AlikeJoins;
source!(AlikeJoins, "the substitution test of Harris on the join table: two labels that stand on one side of joins with enough shared partners, and give one answer with every shared partner, are alike; a join that one of them has and the other lacks is lent to the other, when every label alike to it lends the same answer. A lent join comes after every counted one, so it is used only where no counted join reaches the whole way");

const ALIKE_SHARED: usize = 3;
because!(ALIKE_SHARED, AlikeJoins, "two labels are alike from this many shared partners on, each with one answer: fewer can be chance");

fn alike_joins(joins: &Joins) -> Vec<((u32, u32), u32)> {
    let mut lent: FirstSightTable<(u32, u32), Option<u32>> = FirstSightTable::new();
    for left_side in [true, false] {
        let mut sides: FirstSightTable<u32, FirstSightTable<u32, u32>> = FirstSightTable::new();
        for ((left, right), answer) in joins.table.rows() {
            let (mine, partner) = if left_side { (*left, *right) } else { (*right, *left) };
            sides.at(mine).put(partner, *answer);
        }
        for (label, partners) in sides.rows() {
            for (other, theirs) in sides.rows() {
                if other == label {
                    continue;
                }
                let shared: Vec<&(u32, u32)> = partners.rows().iter().filter(|(partner, _)| theirs.holds(partner)).collect();
                if shared.len() < ALIKE_SHARED || !shared.iter().all(|(partner, answer)| theirs.get(partner) == Some(answer)) {
                    continue;
                }
                for (partner, answer) in theirs.rows() {
                    let key = if left_side { (*label, *partner) } else { (*partner, *label) };
                    if joins.table.holds(&key) {
                        continue;
                    }
                    let held = lent.entry(key, Some(*answer));
                    if *held != Some(*answer) {
                        *held = None;
                    }
                }
            }
        }
    }
    lent.rows().iter().filter_map(|(key, answer)| answer.map(|answer| (*key, answer))).collect()
}
because!(alike_joins, AlikeJoins, "the joins that alike labels lend, on the left side and on the right side of a join, in the order of first sight; a join that two alike labels would lend with different answers is lent by none");

pub(super) fn link_sure(counts: &FirstSightCount<u32>) -> Option<u32> {
    let total: i64 = counts.rows().iter().map(|(_, count)| *count).sum();
    let (answer, hit) = counts.first_most()?;
    (total >= i64::from(LINK_LEAST) && (*hit as f64) >= LINK_SURE_SHARE * total as f64).then_some(*answer)
}
because!(link_sure, LinkJoins, "the answer of an entry when it has enough readings and nearly all of them give the most frequent answer, the first of equals in the order of the lessons; nothing else");

pub(super) fn link_order(left: &(f64, u32, u32), right: &(f64, u32, u32), items: &Symbols) -> Ordering {
    left.0.partial_cmp(&right.0).unwrap_or(Ordering::Equal).then_with(|| items.name(left.1).cmp(items.name(right.1))).then_with(|| items.name(left.2).cmp(items.name(right.2)))
}
because!(link_order, LinkJoins, "the order of two candidates of a round that makes labels one: by their gain, then by the text of the label, then by the text of the answer, so the same candidate wins in every run");
