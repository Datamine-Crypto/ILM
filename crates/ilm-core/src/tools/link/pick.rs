use crate::base::first_sight::FirstSightCount;
use crate::base::symbols::Symbols;
use crate::tools::link::joins::{LINK_LEAST, LINK_SURE_SHARE};
use crate::tools::link::read::{LinkNear, link_search};
use patterns::{because, source};
use std::cmp::Ordering;

pub struct LinkPick;
source!(LinkPick, "a question can hold three things or more, with an answer made of fixed words, things of the question, and at one place one thing of several; the thing at that place is the one that a search from the fixed thing reaches. A link can be read in two directions and one of them can be wrong for a search, so the label whose loss makes the most lessons come out right is shut, again and again, while enough lessons are gained");

pub(super) const PICK_SURE_LESSONS: usize = 20;
because!(PICK_SURE_LESSONS, LinkPick, "the least lessons with three things or more in the question before a form of their answer is looked for, and the least lessons with no way before their answer is used: fewer do not show that an outcome is certain");

const PICK_ROUNDS: usize = 40;
because!(PICK_ROUNDS, LinkPick, "the search for labels to shut, and for labels that are one label, ends after this many rounds: a set has fewer labels to settle");

const PICK_LABELS: usize = 50;
because!(PICK_LABELS, LinkPick, "the labels that the search for shut links tries in a round, the most frequent first: rarer labels change few lessons");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum PickItem {
    Thing(usize),
    Word(String),
}
because!(PickItem, LinkPick, "one place of the answer of a question with three things or more: a thing of the question by its place there, or a fixed word as the answer writes it");

#[derive(Clone, Debug, Default)]
pub(super) struct LinkPicker {
    pub(super) form: Vec<Option<PickItem>>,
    pub(super) picks: Vec<usize>,
    pub(super) start: usize,
    pub(super) shut: Vec<u32>,
}
because!(LinkPicker, LinkPick, "the form of the answer, with nothing at the place of the thing that is picked; the places of the things of the question that can be picked; the place of the thing to start from; and the labels of the links that are shut. reached gives the places of the things that a search from the start reaches");

impl LinkPicker {
    pub(super) fn reached(&self, ends: &[u32], near: &LinkNear, shut: &[u32]) -> Vec<usize> {
        self.picks.iter().copied().filter(|pick| *pick < ends.len() && link_search(near, ends[self.start], ends[*pick], shut).is_some()).collect()
    }
}

pub(super) type PickLesson = (Vec<u32>, LinkNear, Vec<PickItem>);
because!(PickLesson, LinkPick, "one lesson with three things or more in its question: the things of the question, the links of its story, and its answer as places");

pub(super) fn learn_pick(lessons: Vec<PickLesson>, items: &Symbols) -> Option<LinkPicker> {
    if lessons.len() < PICK_SURE_LESSONS {
        return None;
    }
    let mut sizes: FirstSightCount<usize> = FirstSightCount::new();
    for (_, _, answer) in &lessons {
        sizes.add(answer.len(), 1);
    }
    let size = sizes.first_most().map(|(size, _)| *size)?;
    let lessons: Vec<PickLesson> = lessons.into_iter().filter(|(_, _, answer)| answer.len() == size).collect();
    let mut form: Vec<Option<PickItem>> = Vec::with_capacity(size);
    let mut place: Option<usize> = None;
    for at in 0..size {
        let mut counts: FirstSightCount<&PickItem> = FirstSightCount::new();
        for (_, _, answer) in &lessons {
            counts.add(&answer[at], 1);
        }
        let (top, hit) = counts.first_most()?;
        if (*hit as f64) >= LINK_SURE_SHARE * (lessons.len() as f64) {
            form.push(Some((*top).clone()));
        } else if place.is_none() && counts.rows().iter().all(|(item, _)| matches!(item, PickItem::Thing(_))) {
            form.push(None);
            place = Some(at);
        } else {
            return None;
        }
    }
    let place = place?;
    let mut picks: Vec<usize> = lessons.iter().filter_map(|(_, _, answer)| match &answer[place] {
        PickItem::Thing(at) => Some(*at),
        PickItem::Word(_) => None,
    }).collect();
    picks.sort_unstable();
    picks.dedup();
    let fixed = form.iter().find_map(|item| match item {
        Some(PickItem::Thing(at)) => Some(*at),
        _ => None,
    });
    let start = fixed.unwrap_or_else(|| (0..=picks.len()).find(|at| !picks.contains(at)).unwrap_or(picks.len()));
    let mut picker = LinkPicker { form, picks, start, shut: Vec::new() };
    let right = |picker: &LinkPicker, shut: &[u32]| {
        lessons
            .iter()
            .filter(|(ends, near, answer)| picker.start < ends.len() && matches!(&answer[place], PickItem::Thing(at) if picker.reached(ends, near, shut) == [*at]))
            .count()
    };
    let mut seen: FirstSightCount<u32> = FirstSightCount::new();
    for (_, near, _) in &lessons {
        for (_, links) in near.rows() {
            for (_, label) in links {
                seen.add(*label, 1);
            }
        }
    }
    let frequent: Vec<u32> = seen.by_count().iter().take(PICK_LABELS).map(|(label, _)| *label).collect();
    let mut best = right(&picker, &[]);
    for _ in 0..PICK_ROUNDS {
        let mut top: Option<(usize, u32)> = None;
        for label in frequent.iter().filter(|label| !picker.shut.contains(label)) {
            let mut shut = picker.shut.clone();
            shut.push(*label);
            let count = right(&picker, &shut);
            let better = match top {
                None => true,
                Some((held, held_label)) => count > held || (count == held && items.name(*label).cmp(items.name(held_label)) == Ordering::Greater),
            };
            if better {
                top = Some((count, *label));
            }
        }
        let Some((count, label)) = top else { break };
        if count < best + LINK_LEAST as usize {
            break;
        }
        best = count;
        picker.shut.push(label);
    }
    Some(picker)
}
because!(learn_pick, LinkPick, "the form of the answer of the lessons with three things or more in the question, the thing to start from, and the labels to shut; nothing when the lessons have no such form. The answers of the most frequent length count; each place holds one word or one thing of the question in nearly all of them, but for one place that holds things only, which is the place of the pick; the start is the fixed thing of the form, or the first place of a thing that is never picked");
